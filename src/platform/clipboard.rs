//! The clipboard calls of the `Paste` action (owner request 2026-10-30,
//! roadmap new feature 9), and what the paste session decides about the
//! clipboard's formats without Windows.
//!
//! The paste session (`core::executor::paste`) saves the clipboard, puts a
//! phrase on it and puts the saved content back through a [`Clipboard`].
//! The daemon's is [`SYSTEM`]. Its Win32 calls sit in the `win32` module
//! below, which `#[cfg(not(test))]` keeps out of the test build: under test
//! [`SYSTEM`] panics at every call before anything reaches Windows, so no
//! test can read or change the clipboard of the machine that runs it,
//! whatever a guard in front of the call does. The tests hand the session a
//! fake that models Microsoft's rules instead.
//! `the_clipboard_calls_are_compiled_out_of_the_tests` pins that the calls
//! appear nowhere else.
//!
//! Every method runs on the action executor, the thread that creates the
//! session's windows and pumps their messages.

use std::fmt;

use windows::core::Result;

/// `CF_TEXT`: ANSI text, which Windows makes from `CF_UNICODETEXT`. The
/// `windows` crate has the `CF_*` ids only under its `Win32_System_Ole`
/// feature.
pub const CF_TEXT: u32 = 1;
const CF_BITMAP: u32 = 2;
const CF_METAFILEPICT: u32 = 3;
const CF_SYLK: u32 = 4;
const CF_DIF: u32 = 5;
const CF_TIFF: u32 = 6;
/// `CF_OEMTEXT`: OEM text, which Windows makes from `CF_UNICODETEXT`.
pub const CF_OEMTEXT: u32 = 7;
const CF_DIB: u32 = 8;
const CF_PALETTE: u32 = 9;
const CF_PENDATA: u32 = 10;
const CF_RIFF: u32 = 11;
const CF_WAVE: u32 = 12;
/// `CF_UNICODETEXT`: UTF-16 text ending with one NUL, lines ending with
/// CR-LF ("Standard Clipboard Formats").
pub const CF_UNICODETEXT: u32 = 13;
const CF_ENHMETAFILE: u32 = 14;
const CF_HDROP: u32 = 15;
/// `CF_LOCALE`: the locale of the text, which Windows adds to a
/// `CF_UNICODETEXT` that comes without one.
pub const CF_LOCALE: u32 = 16;
const CF_DIBV5: u32 = 17;
const CF_OWNERDISPLAY: u32 = 0x80;
const CF_DSPTEXT: u32 = 0x81;
const CF_DSPBITMAP: u32 = 0x82;
const CF_DSPMETAFILEPICT: u32 = 0x83;
const CF_DSPENHMETAFILE: u32 = 0x8E;
/// The ids `RegisterClipboardFormatW` hands out.
const REGISTERED: std::ops::RangeInclusive<u32> = 0xC000..=0xFFFF;

/// The registered formats whose data is not a copy of the content: the
/// OLE object and link formats, which name a live object of their owner
/// (PLAUSIBLE, synthesis 1.7), compared case-insensitively, as
/// `RegisterClipboardFormatW` compares names.
const OLE_FORMATS: [&str; 9] = [
    "Ole Private Data",
    "Object Descriptor",
    "Link Source",
    "Link Source Descriptor",
    "Embed Source",
    "ObjectLink",
    "OwnerLink",
    "Native",
    "DataObject",
];

/// The registered formats the daemon writes next to a phrase and next to a
/// copy it puts back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    /// "Place any data on the clipboard in this format to prevent all
    /// clipboard formats being included in the clipboard history or
    /// synchronized to the user's other devices" (Clipboard Formats).
    ExcludeFromMonitoring,
    /// A DWORD 0: not in the clipboard history.
    CanIncludeInHistory,
    /// A DWORD 0: not synchronised to the user's other devices.
    CanUploadToCloud,
    /// The format clipboard managers such as Ditto skip (PLAUSIBLE).
    ViewerIgnore,
    /// The daemon's own: this content is a copy the daemon put back, so the
    /// next paste does not take its markers for a private original.
    Restored,
}

impl Marker {
    /// Every marker, in the order the daemon registers and writes them.
    pub const ALL: [Marker; 5] = [
        Marker::ExcludeFromMonitoring,
        Marker::CanIncludeInHistory,
        Marker::CanUploadToCloud,
        Marker::ViewerIgnore,
        Marker::Restored,
    ];

    /// The registered name of the format.
    pub fn name(self) -> &'static str {
        match self {
            Marker::ExcludeFromMonitoring => "ExcludeClipboardContentFromMonitorProcessing",
            Marker::CanIncludeInHistory => "CanIncludeInClipboardHistory",
            Marker::CanUploadToCloud => "CanUploadToCloudClipboard",
            Marker::ViewerIgnore => "Clipboard Viewer Ignore",
            Marker::Restored => "KeySwitchRsRestored",
        }
    }

    /// The marker registered as `name`, compared as Windows compares format
    /// names.
    pub fn named(name: &str) -> Option<Marker> {
        Marker::ALL
            .into_iter()
            .find(|marker| marker.name().eq_ignore_ascii_case(name))
    }
}

/// A message-only window of the action executor, as an integer, since an
/// `HWND` is not `Send`. It owns the clipboard from the daemon's
/// `EmptyClipboard` until the end of its paste session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnerWindow(isize);

impl OwnerWindow {
    /// The window's handle as an integer, as [`Clipboard::owner`] returns
    /// the clipboard's owner.
    pub fn raw(self) -> isize {
        self.0
    }
}

/// What [`Clipboard::read`] found.
#[derive(Debug)]
pub enum Read {
    /// A copy of the format's data.
    Bytes(Zeroed),
    /// The data has this many bytes, more than the limit asked for.
    TooBig(usize),
    /// The copy could not be reserved (critique M8).
    NoMemory,
    /// The format has no data the daemon can read.
    Failed,
}

/// What [`Clipboard::write`] puts on the clipboard. No `Debug`: it is the
/// phrase or a saved copy.
pub enum Payload<'a> {
    /// UTF-16 code units, the phrase's `CF_UNICODETEXT`.
    Units(&'a [u16]),
    /// The bytes of a format, as saved.
    Bytes(&'a [u8]),
}

/// A copy of one clipboard format's data, wiped in place when dropped
/// (critique M8), so that a saved password does not stay behind in freed
/// memory. Its `Debug` shows the length only, and it has no `Clone`.
pub struct Zeroed(Vec<u8>);

impl Zeroed {
    /// Takes `bytes`, which the wipe overwrites where they are: the buffer
    /// never grows, so it never leaves a copy behind.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn wipe(&mut self) {
        self.0.fill(0);
        // The buffer is freed right after the drop; this keeps the
        // compiler from dropping the stores as dead.
        std::hint::black_box(&mut self.0);
    }
}

impl Drop for Zeroed {
    fn drop(&mut self) {
        self.wipe();
    }
}

impl fmt::Debug for Zeroed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Zeroed(<{} bytes>)", self.0.len())
    }
}

/// The clipboard as the paste session uses it. Every method runs on the
/// action executor, and every method but the window, owner, name, register
/// and sequence ones needs the clipboard open by this thread, as Windows
/// does.
pub trait Clipboard {
    /// Creates a message-only window of the calling thread to own the
    /// clipboard.
    fn create_window(&self) -> Result<OwnerWindow>;
    /// Destroys `window`; a failure is logged.
    fn destroy_window(&self, window: OwnerWindow);
    /// The window that owns the clipboard now, if any.
    fn owner(&self) -> Option<isize>;
    /// Opens the clipboard for `window`.
    fn open(&self, window: OwnerWindow) -> Result<()>;
    fn close(&self) -> Result<()>;
    /// Empties the clipboard and makes the opening window its owner.
    fn empty(&self) -> Result<()>;
    /// The formats on the clipboard, in the order Windows enumerates them,
    /// into `into`, which it clears first.
    fn formats(&self, into: &mut Vec<u32>);
    /// The name of the registered format `format` into `into`, or `false`.
    fn format_name(&self, format: u32, into: &mut String) -> bool;
    /// A copy of the data of `format`, if it has at most `limit` bytes.
    fn read(&self, format: u32, limit: usize) -> Read;
    /// Puts `data` on the clipboard as `format`.
    fn write(&self, format: u32, data: Payload<'_>) -> Result<()>;
    /// The id of `marker`'s registered format, or `None`.
    fn register(&self, marker: Marker) -> Option<u32>;
    /// The clipboard's sequence number, for a diagnostic only.
    fn sequence_number(&self) -> u32;
}

/// The daemon's clipboard: the Win32 calls of the `win32` module outside
/// tests, a panic inside them.
pub struct System;

/// The clipboard the daemon uses.
pub const SYSTEM: System = System;

#[cfg(not(test))]
mod win32 {
    use windows::Win32::{
        Foundation::{GlobalFree, HANDLE, HGLOBAL, HWND},
        System::{
            DataExchange::{
                CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
                GetClipboardFormatNameW, GetClipboardOwner, GetClipboardSequenceNumber,
                OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
            },
            Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock},
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, HWND_MESSAGE, WINDOW_EX_STYLE, WINDOW_STYLE,
        },
    };
    use windows::core::{PCWSTR, Result, w};

    use super::{Clipboard, Marker, OwnerWindow, Payload, Read, System, Zeroed};

    /// How many formats an enumeration reads at most, so that a clipboard
    /// that never ends its enumeration cannot hold up the executor.
    const MAX_FORMATS: usize = 1024;

    fn hwnd(window: OwnerWindow) -> HWND {
        HWND(window.0 as *mut core::ffi::c_void)
    }

    /// A block of `GlobalAlloc` memory, freed when dropped unless the
    /// clipboard took it.
    struct Block(HGLOBAL);

    impl Drop for Block {
        fn drop(&mut self) {
            // SAFETY: `self.0` came from GlobalAlloc in `write`, is unlocked,
            // and the clipboard did not take it (`write` forgets the guard
            // once SetClipboardData took it), so this frees it once. The
            // wrapper's Err is the documented success (a NULL return).
            let _ = unsafe { GlobalFree(Some(self.0)) };
        }
    }

    impl Clipboard for System {
        fn create_window(&self) -> Result<OwnerWindow> {
            // SAFETY: "Message" is the system class of message-only windows,
            // so no class of ours is registered; the window name is null and
            // no creation data is passed. HWND_MESSAGE makes a window that
            // no other process enumerates or sends input to.
            let window = unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("Message"),
                    None,
                    WINDOW_STYLE::default(),
                    0,
                    0,
                    0,
                    0,
                    Some(HWND_MESSAGE),
                    None,
                    None,
                    None,
                )
            }?;
            Ok(OwnerWindow(window.0 as isize))
        }

        fn destroy_window(&self, window: OwnerWindow) {
            // SAFETY: `window` came from `create_window` on this thread, the
            // action executor, which the session never leaves, as
            // DestroyWindow requires; the session destroys each window once.
            if let Err(error) = unsafe { DestroyWindow(hwnd(window)) } {
                log::warn!("paste: could not destroy the clipboard window ({error})");
            }
        }

        fn owner(&self) -> Option<isize> {
            // SAFETY: GetClipboardOwner takes no arguments and reads no
            // memory of ours; a clipboard without an owner gives an error.
            let owner = unsafe { GetClipboardOwner() };
            owner.ok().map(|window| window.0 as isize)
        }

        fn open(&self, window: OwnerWindow) -> Result<()> {
            // SAFETY: `window` is a live window of this thread that the
            // session made; the call takes it by value and fails while
            // another window has the clipboard open.
            unsafe { OpenClipboard(Some(hwnd(window))) }
        }

        fn close(&self) -> Result<()> {
            // SAFETY: CloseClipboard takes no arguments; the session calls it
            // only after its own open succeeded.
            unsafe { CloseClipboard() }
        }

        fn empty(&self) -> Result<()> {
            // SAFETY: EmptyClipboard takes no arguments; the session calls it
            // only while its window has the clipboard open, so the window
            // becomes the owner.
            unsafe { EmptyClipboard() }
        }

        fn formats(&self, into: &mut Vec<u32>) {
            into.clear();
            let mut format = 0;
            while into.len() < MAX_FORMATS {
                // SAFETY: EnumClipboardFormats takes the previous id by value
                // and reads no memory of ours; it returns 0 at the end, or
                // when this thread does not have the clipboard open.
                format = unsafe { EnumClipboardFormats(format) };
                if format == 0 {
                    break;
                }
                into.push(format);
            }
        }

        fn format_name(&self, format: u32, into: &mut String) -> bool {
            let mut buffer = [0_u16; 256];
            // SAFETY: `buffer` is a local array that outlives the call, which
            // writes at most its length (the wrapper passes it) and returns
            // how many units it wrote, 0 for a predefined or unknown id.
            let length = unsafe { GetClipboardFormatNameW(format, &mut buffer) };
            let Some(units) = usize::try_from(length)
                .ok()
                .filter(|&length| length > 0)
                .and_then(|length| buffer.get(..length))
            else {
                return false;
            };
            into.clear();
            into.extend(
                char::decode_utf16(units.iter().copied())
                    .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER)),
            );
            true
        }

        fn read(&self, format: u32, limit: usize) -> Read {
            // SAFETY: the session reads only while its window has the
            // clipboard open; the call takes the id by value and returns a
            // handle the clipboard keeps, which is copied before the close.
            let Ok(handle) = (unsafe { GetClipboardData(format) }) else {
                return Read::Failed;
            };
            let block = HGLOBAL(handle.0);
            // SAFETY: `block` is the handle the clipboard returned for a
            // format the session reads as global memory (`classify`); the
            // call only reads its size and returns 0 for a handle that is
            // not a global block.
            let size = unsafe { GlobalSize(block) };
            if size == 0 {
                return Read::Failed;
            }
            if size > limit {
                return Read::TooBig(size);
            }
            let mut bytes = Vec::new();
            if bytes.try_reserve_exact(size).is_err() {
                return Read::NoMemory;
            }
            // SAFETY: as for GlobalSize; the lock is undone below on every
            // path that got a pointer.
            let data = unsafe { GlobalLock(block) };
            if data.is_null() {
                return Read::Failed;
            }
            // SAFETY: GlobalLock returned the block's first byte, valid for
            // the `size` bytes GlobalSize gave until GlobalUnlock; the bytes
            // are copied into `bytes`, whose exact reserve holds them, before
            // the unlock, and the slice does not outlive this statement.
            bytes.extend_from_slice(unsafe { std::slice::from_raw_parts(data.cast::<u8>(), size) });
            // SAFETY: balances the GlobalLock above on the same handle. Its
            // Err with code 0 is the documented success of the last unlock.
            let _ = unsafe { GlobalUnlock(block) };
            Read::Bytes(Zeroed::new(bytes))
        }

        fn write(&self, format: u32, data: Payload<'_>) -> Result<()> {
            let (source, size) = match data {
                Payload::Units(units) => (units.as_ptr().cast::<u8>(), size_of_val(units)),
                Payload::Bytes(bytes) => (bytes.as_ptr(), bytes.len()),
            };
            // SAFETY: GlobalAlloc takes the flags and the size by value and
            // reads no memory of ours; a moveable block is what the clipboard
            // takes.
            let block = Block(unsafe { GlobalAlloc(GMEM_MOVEABLE, size) }?);
            // SAFETY: `block.0` is the block just allocated and not locked
            // yet; the lock is undone below.
            let target = unsafe { GlobalLock(block.0) };
            if target.is_null() {
                return Err(windows::core::Error::from_thread());
            }
            // SAFETY: `target` points to at least `size` writable bytes of
            // the locked block, `source` to `size` bytes of the live slice
            // `data` borrows, and the two never overlap: one is this
            // process's heap, the other the new global block.
            unsafe { std::ptr::copy_nonoverlapping(source, target.cast::<u8>(), size) };
            // SAFETY: balances the GlobalLock above on the same handle. Its
            // Err with code 0 is the documented success of the last unlock.
            let _ = unsafe { GlobalUnlock(block.0) };
            // SAFETY: the session writes only while its window has the
            // clipboard open and owns it (after its own EmptyClipboard);
            // `block.0` is an unlocked moveable block, which the system owns
            // once the call succeeds.
            unsafe { SetClipboardData(format, Some(HANDLE(block.0.0))) }?;
            std::mem::forget(block);
            Ok(())
        }

        fn register(&self, marker: Marker) -> Option<u32> {
            let name: Vec<u16> = marker
                .name()
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            // SAFETY: `name` is a NUL-terminated UTF-16 buffer that lives
            // across the call, which only reads it.
            let id = unsafe { RegisterClipboardFormatW(PCWSTR(name.as_ptr())) };
            (id != 0).then_some(id)
        }

        fn sequence_number(&self) -> u32 {
            // SAFETY: GetClipboardSequenceNumber takes no arguments and reads
            // no memory of ours.
            unsafe { GetClipboardSequenceNumber() }
        }
    }
}

/// What every call of the system clipboard says under test.
#[cfg(test)]
const UNDER_TEST: &str =
    "a test reached the system clipboard, which no test build calls (lane feat-clipboard-paste)";

/// Under test no call reaches Windows: a test that gets here has a defect,
/// and fails at the panic instead of reading or changing the clipboard of
/// the machine that runs it.
#[cfg(test)]
impl Clipboard for System {
    fn create_window(&self) -> Result<OwnerWindow> {
        unreachable!("{UNDER_TEST}")
    }

    fn destroy_window(&self, _window: OwnerWindow) {
        unreachable!("{UNDER_TEST}")
    }

    fn owner(&self) -> Option<isize> {
        unreachable!("{UNDER_TEST}")
    }

    fn open(&self, _window: OwnerWindow) -> Result<()> {
        unreachable!("{UNDER_TEST}")
    }

    fn close(&self) -> Result<()> {
        unreachable!("{UNDER_TEST}")
    }

    fn empty(&self) -> Result<()> {
        unreachable!("{UNDER_TEST}")
    }

    fn formats(&self, _into: &mut Vec<u32>) {
        unreachable!("{UNDER_TEST}")
    }

    fn format_name(&self, _format: u32, _into: &mut String) -> bool {
        unreachable!("{UNDER_TEST}")
    }

    fn read(&self, _format: u32, _limit: usize) -> Read {
        unreachable!("{UNDER_TEST}")
    }

    fn write(&self, _format: u32, _data: Payload<'_>) -> Result<()> {
        unreachable!("{UNDER_TEST}")
    }

    fn register(&self, _marker: Marker) -> Option<u32> {
        unreachable!("{UNDER_TEST}")
    }

    fn sequence_number(&self) -> u32 {
        unreachable!("{UNDER_TEST}")
    }
}

/// What the session does with a format of the clipboard it saves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Saved and put back.
    Keep,
    /// `CF_DIB` or `CF_DIBV5`: the first one enumerated is the one on the
    /// clipboard, and Windows makes the other from it ("the system first
    /// enumerates the format that is on the clipboard, followed by the
    /// formats to which it can be converted", Clipboard Formats).
    Bitmap,
    /// Not saved: Windows makes it from another format, its data is not a
    /// global memory block, or the daemon writes it itself.
    Skip,
}

/// The class of `format`, whose registered name is `name` (`None` for a
/// predefined id, or a registered one whose name could not be read).
///
/// `CF_GDIOBJFIRST` to `CF_GDIOBJLAST` (0x300-0x3FF) are skipped as the
/// private range is: Microsoft's "Clipboard Formats" says the system frees
/// their handles with `GlobalFree`, its "Standard Clipboard Formats" says
/// it does not (critique M2), so their handles may not be blocks of this
/// process.
pub fn classify(format: u32, name: Option<&str>) -> Class {
    match format {
        CF_UNICODETEXT | CF_SYLK | CF_DIF | CF_TIFF | CF_PENDATA | CF_RIFF | CF_WAVE | CF_HDROP
        | CF_LOCALE | CF_DSPTEXT => Class::Keep,
        CF_DIB | CF_DIBV5 => Class::Bitmap,
        CF_TEXT | CF_OEMTEXT | CF_BITMAP | CF_PALETTE | CF_DSPBITMAP | CF_METAFILEPICT
        | CF_ENHMETAFILE | CF_DSPMETAFILEPICT | CF_DSPENHMETAFILE | CF_OWNERDISPLAY => Class::Skip,
        _ if REGISTERED.contains(&format) => match name {
            Some(name)
                if Marker::named(name).is_none()
                    && !OLE_FORMATS.iter().any(|ole| ole.eq_ignore_ascii_case(name)) =>
            {
                Class::Keep
            }
            _ => Class::Skip,
        },
        _ => Class::Skip,
    }
}

/// The formats of an enumeration, each with its registered name, that the
/// session saves, in enumeration order: the [kept](Class::Keep) ones and the
/// first [bitmap](Class::Bitmap) one.
pub fn kept_formats<'a>(enumerated: impl IntoIterator<Item = (u32, Option<&'a str>)>) -> Vec<u32> {
    let mut kept = Vec::new();
    let mut bitmap = false;
    for (format, name) in enumerated {
        match classify(format, name) {
            Class::Keep => kept.push(format),
            Class::Bitmap if !bitmap => {
                bitmap = true;
                kept.push(format);
            }
            Class::Bitmap | Class::Skip => {}
        }
    }
    kept
}

/// Whether `enumerated` holds a format that the daemon did not write (the
/// ids in `written`) and that Windows does not make from its text: another
/// program added it without emptying the clipboard (critique M2).
pub fn foreign_format_present(enumerated: &[u32], written: &[u32]) -> bool {
    enumerated.iter().any(|format| {
        !written.contains(format) && !matches!(*format, CF_TEXT | CF_OEMTEXT | CF_LOCALE)
    })
}

/// What a [`FakeClipboard`] was asked, in order.
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Call {
    Owner,
    Open(isize),
    Close,
    Empty,
    Formats,
    Name(u32),
    /// A read of a format with the limit asked for.
    Read(u32, usize),
    Write(u32),
    Register(Marker),
    SequenceNumber,
}

/// A failure a [`FakeClipboard`] can be told to report.
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Fail {
    /// `create_window`.
    Create,
    /// `empty`.
    Empty,
    /// A write of `CF_UNICODETEXT`.
    WriteText,
    /// A write of a registered marker.
    WriteMarker,
    /// Every registration.
    Register,
}

/// The windows a [`FakeClipboard`] made and destroyed, in order.
#[cfg(test)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Windows {
    pub(crate) created: Vec<isize>,
    pub(crate) destroyed: Vec<isize>,
}

/// The clock a [`FakeClipboard`] moves when a read costs time: the offset
/// of a test's fake clock.
#[cfg(test)]
pub(crate) type FakeClock = std::sync::Arc<std::sync::Mutex<std::time::Duration>>;

/// The state of a [`FakeClipboard`].
#[cfg(test)]
#[derive(Default)]
struct FakeState {
    /// The formats on the clipboard, in the order they were put there.
    content: Vec<(u32, Vec<u8>)>,
    owner: Option<isize>,
    open_by: Option<isize>,
    names: Vec<(u32, String)>,
    registered: Vec<(Marker, u32)>,
    /// How many of the next opens fail as if another program held the
    /// clipboard open.
    busy_opens: u32,
    failures: Vec<Fail>,
    panic_at: Option<Call>,
    read_costs: Vec<(u32, std::time::Duration)>,
    clock: Option<FakeClock>,
    /// Sizes the next reads of a format report instead of its data's.
    declared: Vec<(u32, usize)>,
    reserve_failures: Vec<u32>,
    calls: Vec<Call>,
    windows: Windows,
    sequence: u32,
}

#[cfg(test)]
impl FakeState {
    fn record(&mut self, call: Call) {
        self.calls.push(call);
        if self.panic_at == Some(call) {
            panic!("the fake clipboard panics at {call:?}");
        }
    }

    fn fails(&self, fail: Fail) -> bool {
        self.failures.contains(&fail)
    }

    fn has(&self, format: u32) -> bool {
        self.content.iter().any(|(each, _)| *each == format)
    }

    /// The formats as Windows enumerates them: those put there, then the
    /// ones it makes from `CF_UNICODETEXT`.
    fn enumeration(&self) -> Vec<u32> {
        let mut formats: Vec<u32> = self.content.iter().map(|(format, _)| *format).collect();
        if self.has(CF_UNICODETEXT) {
            for synthesized in [CF_LOCALE, CF_TEXT, CF_OEMTEXT] {
                if !self.has(synthesized) {
                    formats.push(synthesized);
                }
            }
        }
        formats
    }

    fn data(&self, format: u32) -> Option<Vec<u8>> {
        if let Some((_, bytes)) = self.content.iter().find(|(each, _)| *each == format) {
            return Some(bytes.clone());
        }
        let text = self.has(CF_UNICODETEXT);
        match format {
            CF_LOCALE if text => Some(vec![0x09, 0x04, 0, 0]),
            CF_TEXT | CF_OEMTEXT if text => Some(b"?\0".to_vec()),
            _ => None,
        }
    }

    fn is_marker(&self, format: u32) -> bool {
        self.registered.iter().any(|(_, id)| *id == format)
    }
}

/// A clipboard that follows Microsoft's rules for one process and records
/// every call, for the tests of the paste session. Nothing reaches Windows.
/// Clones share the clipboard, so a test keeps one and hands the other to
/// the session.
///
/// `open` fails while another window has it open or while busy opens are
/// left; `empty` needs this window's open and makes it the owner; `write`
/// needs the open and an owner; the enumeration lists the formats put there,
/// then `16, 1, 7` when `CF_UNICODETEXT` is there.
#[cfg(test)]
#[derive(Clone, Default)]
pub(crate) struct FakeClipboard(std::sync::Arc<std::sync::Mutex<FakeState>>);

#[cfg(test)]
impl FakeClipboard {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn state(&self) -> std::sync::MutexGuard<'_, FakeState> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The clipboard holds `formats`, put there by `owner`.
    pub(crate) fn with_content(self, owner: isize, formats: &[(u32, &[u8])]) -> Self {
        self.foreign_copy(owner, formats);
        self.state().calls.clear();
        self
    }

    /// The registered format `id` is named `name`.
    pub(crate) fn with_name(self, id: u32, name: &str) -> Self {
        self.state().names.push((id, name.to_owned()));
        self
    }

    /// The next `count` opens fail.
    pub(crate) fn with_busy_opens(self, count: u32) -> Self {
        self.busy_opens(count);
        self
    }

    pub(crate) fn with_failure(self, fail: Fail) -> Self {
        self.state().failures.push(fail);
        self
    }

    /// The fake panics when it is asked `call`, after recording it.
    pub(crate) fn with_panic_at(self, call: Call) -> Self {
        self.state().panic_at = Some(call);
        self
    }

    /// A read of `format` moves `clock` on by `cost`.
    pub(crate) fn with_read_cost(
        self,
        format: u32,
        cost: std::time::Duration,
        clock: FakeClock,
    ) -> Self {
        {
            let mut state = self.state();
            state.read_costs.push((format, cost));
            state.clock = Some(clock);
        }
        self
    }

    /// A read of `format` finds `size` bytes, whatever its data holds.
    pub(crate) fn with_declared_size(self, format: u32, size: usize) -> Self {
        self.state().declared.push((format, size));
        self
    }

    /// The copy of `format` cannot be reserved.
    pub(crate) fn with_reserve_failure(self, format: u32) -> Self {
        self.state().reserve_failures.push(format);
        self
    }

    /// The next `count` opens fail.
    pub(crate) fn busy_opens(&self, count: u32) {
        self.state().busy_opens = count;
    }

    /// Another program copies `formats`: it empties the clipboard and owns
    /// it.
    pub(crate) fn foreign_copy(&self, owner: isize, formats: &[(u32, &[u8])]) {
        let mut state = self.state();
        state.content = formats
            .iter()
            .map(|(format, bytes)| (*format, bytes.to_vec()))
            .collect();
        state.owner = Some(owner);
        state.sequence += 1;
    }

    /// Another program adds `format` without emptying the clipboard.
    pub(crate) fn foreign_add(&self, format: u32, bytes: &[u8]) {
        let mut state = self.state();
        state.content.push((format, bytes.to_vec()));
        state.sequence += 1;
    }

    /// The formats on the clipboard, in the order they were put there.
    pub(crate) fn content(&self) -> Vec<(u32, Vec<u8>)> {
        self.state().content.clone()
    }

    pub(crate) fn current_owner(&self) -> Option<isize> {
        self.state().owner
    }

    /// Every call so far, registrations included.
    pub(crate) fn calls(&self) -> Vec<Call> {
        self.state().calls.clone()
    }

    pub(crate) fn is_open(&self) -> bool {
        self.state().open_by.is_some()
    }

    pub(crate) fn windows(&self) -> Windows {
        self.state().windows.clone()
    }

    /// The id the fake registered for `marker`, if it did.
    pub(crate) fn marker_id(&self, marker: Marker) -> Option<u32> {
        self.state()
            .registered
            .iter()
            .find(|(each, _)| *each == marker)
            .map(|(_, id)| *id)
    }
}

#[cfg(test)]
impl Clipboard for FakeClipboard {
    fn create_window(&self) -> Result<OwnerWindow> {
        let mut state = self.state();
        if state.fails(Fail::Create) {
            return Err(windows::Win32::Foundation::E_OUTOFMEMORY.into());
        }
        let window = 0x1001 + state.windows.created.len() as isize;
        state.windows.created.push(window);
        Ok(OwnerWindow(window))
    }

    fn destroy_window(&self, window: OwnerWindow) {
        self.state().windows.destroyed.push(window.0);
    }

    fn owner(&self) -> Option<isize> {
        let mut state = self.state();
        state.record(Call::Owner);
        state.owner
    }

    fn open(&self, window: OwnerWindow) -> Result<()> {
        let mut state = self.state();
        state.record(Call::Open(window.0));
        if state.busy_opens > 0 {
            state.busy_opens -= 1;
            return Err(windows::Win32::Foundation::E_ACCESSDENIED.into());
        }
        match state.open_by {
            Some(other) if other != window.0 => {
                Err(windows::Win32::Foundation::E_ACCESSDENIED.into())
            }
            _ => {
                state.open_by = Some(window.0);
                Ok(())
            }
        }
    }

    fn close(&self) -> Result<()> {
        let mut state = self.state();
        state.record(Call::Close);
        match state.open_by.take() {
            Some(_) => Ok(()),
            None => Err(windows::Win32::Foundation::E_ACCESSDENIED.into()),
        }
    }

    fn empty(&self) -> Result<()> {
        let mut state = self.state();
        state.record(Call::Empty);
        let Some(opener) = state.open_by else {
            return Err(windows::Win32::Foundation::E_ACCESSDENIED.into());
        };
        if state.fails(Fail::Empty) {
            return Err(windows::Win32::Foundation::E_FAIL.into());
        }
        state.content.clear();
        state.owner = Some(opener);
        state.sequence += 1;
        Ok(())
    }

    fn formats(&self, into: &mut Vec<u32>) {
        let mut state = self.state();
        state.record(Call::Formats);
        into.clear();
        if state.open_by.is_some() {
            into.extend(state.enumeration());
        }
    }

    fn format_name(&self, format: u32, into: &mut String) -> bool {
        let mut state = self.state();
        state.record(Call::Name(format));
        let registered = state
            .registered
            .iter()
            .find(|(_, id)| *id == format)
            .map(|(marker, _)| marker.name().to_owned());
        let named = state
            .names
            .iter()
            .find(|(id, _)| *id == format)
            .map(|(_, name)| name.clone());
        match registered.or(named) {
            Some(name) => {
                *into = name;
                true
            }
            None => false,
        }
    }

    fn read(&self, format: u32, limit: usize) -> Read {
        let mut state = self.state();
        state.record(Call::Read(format, limit));
        if state.open_by.is_none() {
            return Read::Failed;
        }
        if let Some((_, cost)) = state.read_costs.iter().find(|(each, _)| *each == format)
            && let Some(clock) = &state.clock
        {
            *clock
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) += *cost;
        }
        let Some(bytes) = state.data(format) else {
            return Read::Failed;
        };
        let size = state
            .declared
            .iter()
            .find(|(each, _)| *each == format)
            .map_or(bytes.len(), |(_, size)| *size);
        if size == 0 {
            return Read::Failed;
        }
        if size > limit {
            return Read::TooBig(size);
        }
        if state.reserve_failures.contains(&format) {
            return Read::NoMemory;
        }
        Read::Bytes(Zeroed::new(bytes))
    }

    fn write(&self, format: u32, data: Payload<'_>) -> Result<()> {
        let mut state = self.state();
        state.record(Call::Write(format));
        if state.open_by.is_none() || state.owner.is_none() || state.owner != state.open_by {
            return Err(windows::Win32::Foundation::E_ACCESSDENIED.into());
        }
        if (format == CF_UNICODETEXT && state.fails(Fail::WriteText))
            || (state.is_marker(format) && state.fails(Fail::WriteMarker))
        {
            return Err(windows::Win32::Foundation::E_OUTOFMEMORY.into());
        }
        let bytes = match data {
            Payload::Units(units) => units.iter().flat_map(|unit| unit.to_le_bytes()).collect(),
            Payload::Bytes(bytes) => bytes.to_vec(),
        };
        state.content.retain(|(each, _)| *each != format);
        state.content.push((format, bytes));
        Ok(())
    }

    fn register(&self, marker: Marker) -> Option<u32> {
        let mut state = self.state();
        state.record(Call::Register(marker));
        if state.fails(Fail::Register) {
            return None;
        }
        if let Some((_, id)) = state.registered.iter().find(|(each, _)| *each == marker) {
            return Some(*id);
        }
        let id = 0xC100 + state.registered.len() as u32;
        state.registered.push((marker, id));
        Some(id)
    }

    fn sequence_number(&self) -> u32 {
        let mut state = self.state();
        state.record(Call::SequenceNumber);
        state.sequence
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::path::Path;

    use super::*;
    use crate::scan::{cfg_not_test_block, code_of, doc_code_lines, scanned_sources};

    #[test]
    fn the_system_clipboard_panics_at_every_call_under_test() {
        // Whatever a caller's guard does, the test build's system clipboard
        // stops the call before Windows sees it.
        let window = OwnerWindow(0x1234);
        let calls: [(&str, &dyn Fn()); 12] = [
            ("create_window", &|| drop(SYSTEM.create_window())),
            ("destroy_window", &|| SYSTEM.destroy_window(window)),
            ("owner", &|| {
                let _ = SYSTEM.owner();
            }),
            ("open", &|| drop(SYSTEM.open(window))),
            ("close", &|| drop(SYSTEM.close())),
            ("empty", &|| drop(SYSTEM.empty())),
            ("formats", &|| SYSTEM.formats(&mut Vec::new())),
            ("format_name", &|| {
                let _ = SYSTEM.format_name(0xC000, &mut String::new());
            }),
            ("read", &|| drop(SYSTEM.read(CF_UNICODETEXT, 16))),
            ("write", &|| {
                drop(SYSTEM.write(CF_UNICODETEXT, Payload::Units(&[0])))
            }),
            ("register", &|| {
                let _ = SYSTEM.register(Marker::Restored);
            }),
            ("sequence_number", &|| {
                let _ = SYSTEM.sequence_number();
            }),
        ];
        for (name, call) in calls {
            assert!(
                catch_unwind(AssertUnwindSafe(call)).is_err(),
                "{name} returned under test"
            );
        }
    }

    #[test]
    fn the_formats_are_classified_as_the_table_says() {
        let rows: &[(u32, Option<&str>, Class)] = &[
            (CF_UNICODETEXT, None, Class::Keep),
            (CF_TEXT, None, Class::Skip),
            (CF_OEMTEXT, None, Class::Skip),
            (CF_DIB, None, Class::Bitmap),
            (CF_DIBV5, None, Class::Bitmap),
            (CF_BITMAP, None, Class::Skip),
            (CF_PALETTE, None, Class::Skip),
            (CF_DSPBITMAP, None, Class::Skip),
            (CF_METAFILEPICT, None, Class::Skip),
            (CF_ENHMETAFILE, None, Class::Skip),
            (CF_DSPMETAFILEPICT, None, Class::Skip),
            (CF_DSPENHMETAFILE, None, Class::Skip),
            (CF_OWNERDISPLAY, None, Class::Skip),
            (0x200, None, Class::Skip),
            (0x2FF, None, Class::Skip),
            (0x300, None, Class::Skip),
            (0x3FF, None, Class::Skip),
            (CF_SYLK, None, Class::Keep),
            (CF_DIF, None, Class::Keep),
            (CF_TIFF, None, Class::Keep),
            (CF_PENDATA, None, Class::Keep),
            (CF_RIFF, None, Class::Keep),
            (CF_WAVE, None, Class::Keep),
            (CF_HDROP, None, Class::Keep),
            (CF_LOCALE, None, Class::Keep),
            (CF_DSPTEXT, None, Class::Keep),
            (0xC123, Some("html format"), Class::Keep),
            (0xC124, Some("Rich Text Format"), Class::Keep),
            (0xC125, Some("OLE PRIVATE DATA"), Class::Skip),
            (0xC126, Some("Object Descriptor"), Class::Skip),
            (0xC127, Some("Link Source"), Class::Skip),
            (0xC128, Some("Link Source Descriptor"), Class::Skip),
            (0xC129, Some("Embed Source"), Class::Skip),
            (0xC12A, Some("ObjectLink"), Class::Skip),
            (0xC12B, Some("OwnerLink"), Class::Skip),
            (0xC12C, Some("Native"), Class::Skip),
            (0xC12D, Some("DataObject"), Class::Skip),
            (0xC12E, None, Class::Skip),
            (0x0042, None, Class::Skip),
            (0x0042, Some("not registered"), Class::Skip),
        ];
        for &(format, name, class) in rows {
            assert_eq!(classify(format, name), class, "{format:#x} {name:?}");
        }
        for marker in Marker::ALL {
            assert_eq!(
                classify(0xC200, Some(marker.name())),
                Class::Skip,
                "{marker:?}"
            );
            assert_eq!(
                classify(0xC200, Some(&marker.name().to_ascii_uppercase())),
                Class::Skip,
                "{marker:?} in capitals"
            );
        }
    }

    #[test]
    fn the_second_bitmap_format_is_the_synthesized_one() {
        assert_eq!(kept_formats([(CF_DIBV5, None), (CF_DIB, None)]), [CF_DIBV5]);
        assert_eq!(
            kept_formats([(CF_DIB, None), (CF_DIBV5, None), (CF_BITMAP, None)]),
            [CF_DIB]
        );
        assert_eq!(
            kept_formats([
                (0xC010, Some("HTML Format")),
                (CF_UNICODETEXT, None),
                (CF_LOCALE, None),
                (CF_TEXT, None),
            ]),
            [0xC010, CF_UNICODETEXT, CF_LOCALE],
            "enumeration order is kept"
        );
    }

    #[test]
    fn only_the_synthesized_text_and_locale_formats_are_allowed_besides_what_was_written() {
        let written = [CF_UNICODETEXT, 0xC100, 0xC101, 0xC102, 0xC103];
        let mut enumerated = written.to_vec();
        enumerated.extend([CF_LOCALE, CF_TEXT, CF_OEMTEXT]);
        assert!(!foreign_format_present(&enumerated, &written));
        assert!(foreign_format_present(
            &[&enumerated[..], &[0xC200]].concat(),
            &written
        ));
        assert!(foreign_format_present(
            &[&enumerated[..], &[CF_DIB]].concat(),
            &written
        ));
        assert!(
            !foreign_format_present(&[CF_UNICODETEXT], &written),
            "a marker missing is no foreign format"
        );
    }

    #[test]
    fn a_saved_buffer_is_wiped_in_place() {
        let mut saved = Zeroed::new(vec![0xAB; 64]);
        let pointer = saved.bytes().as_ptr();
        let capacity = saved.0.capacity();
        assert_eq!(format!("{saved:?}"), "Zeroed(<64 bytes>)");

        saved.wipe();

        assert!(
            saved.bytes().iter().all(|&byte| byte == 0),
            "{:?}",
            saved.bytes()
        );
        assert_eq!(saved.bytes().as_ptr(), pointer, "wiped where it was");
        assert_eq!(saved.0.capacity(), capacity);
        assert_eq!(saved.len(), 64);
    }

    #[test]
    fn the_fake_clipboard_follows_the_ownership_rules() {
        let ours = OwnerWindow(0x1001);
        let other = OwnerWindow(0x2002);
        let fake = FakeClipboard::new().with_content(0x77, &[(CF_UNICODETEXT, b"a\0\0\0")]);

        assert!(fake.empty().is_err(), "empty without an open");
        fake.open(other).expect("the other window opens");
        assert!(fake.open(ours).is_err(), "another window has it open");
        assert!(
            fake.write(CF_UNICODETEXT, Payload::Units(&[0x61, 0]))
                .is_err(),
            "the other window does not own it"
        );
        fake.close().expect("the other window closes");

        fake.open(ours).expect("ours opens");
        let mut formats = Vec::new();
        fake.formats(&mut formats);
        assert_eq!(formats, [CF_UNICODETEXT, CF_LOCALE, CF_TEXT, CF_OEMTEXT]);
        fake.empty().expect("ours empties");
        assert_eq!(fake.current_owner(), Some(ours.raw()), "the opener owns it");
        fake.write(CF_UNICODETEXT, Payload::Units(&[0x62, 0]))
            .expect("the owner writes");
        fake.close().expect("ours closes");
        assert_eq!(fake.content(), [(CF_UNICODETEXT, vec![0x62, 0, 0, 0])]);

        fake.foreign_copy(0x99, &[]);
        fake.open(ours).expect("ours opens again");
        assert!(fake.empty().is_ok());
        fake.close().expect("closes");
        let null_owner = FakeClipboard::new();
        null_owner.open(ours).expect("opens");
        assert!(
            null_owner
                .write(CF_UNICODETEXT, Payload::Units(&[0]))
                .is_err(),
            "a write with no owner fails"
        );
    }

    #[test]
    fn the_fake_clipboard_reports_what_it_was_told_to() {
        let clock = FakeClock::default();
        let ours = OwnerWindow(0x1001);
        let fake = FakeClipboard::new()
            .with_content(0x77, &[(0xC010, b"<b>x</b>"), (CF_DIB, b"bits")])
            .with_name(0xC010, "HTML Format")
            .with_busy_opens(1)
            .with_read_cost(0xC010, std::time::Duration::from_millis(30), clock.clone())
            .with_declared_size(CF_DIB, 1 << 30)
            .with_failure(Fail::WriteMarker);

        assert!(fake.open(ours).is_err(), "the busy open");
        fake.open(ours).expect("the next open");
        assert!(fake.is_open());
        let mut name = String::new();
        assert!(fake.format_name(0xC010, &mut name));
        assert_eq!(name, "HTML Format");
        assert!(
            !fake.format_name(0xC011, &mut name),
            "an unknown id has no name"
        );
        assert!(
            matches!(fake.read(0xC010, 64), Read::Bytes(bytes) if bytes.bytes() == b"<b>x</b>")
        );
        assert_eq!(
            *clock.lock().expect("the clock"),
            std::time::Duration::from_millis(30)
        );
        assert!(matches!(fake.read(CF_DIB, 1 << 20), Read::TooBig(size) if size == 1 << 30));
        assert!(matches!(fake.read(0xC0FF, 64), Read::Failed));

        let marker = fake.register(Marker::Restored).expect("registers");
        assert_eq!(
            fake.register(Marker::Restored),
            Some(marker),
            "once per name"
        );
        assert_eq!(fake.marker_id(Marker::Restored), Some(marker));
        fake.empty().expect("empties");
        assert!(
            fake.write(marker, Payload::Bytes(&[0; 4])).is_err(),
            "the marker write fails"
        );
        fake.write(CF_UNICODETEXT, Payload::Units(&[0x61, 0]))
            .expect("the text write");
        fake.close().expect("closes");
        assert!(!fake.is_open());
        fake.foreign_add(0xC300, b"added");
        assert_eq!(
            fake.content(),
            [
                (CF_UNICODETEXT, vec![0x61, 0, 0, 0]),
                (0xC300, b"added".to_vec())
            ]
        );
        assert_eq!(fake.calls()[..2], [Call::Open(0x1001), Call::Open(0x1001)]);

        let window = fake.create_window().expect("a window");
        fake.destroy_window(window);
        assert_eq!(
            fake.windows(),
            Windows {
                created: vec![0x1001],
                destroyed: vec![0x1001]
            }
        );
        let failing = FakeClipboard::new()
            .with_failure(Fail::Create)
            .with_failure(Fail::Empty)
            .with_failure(Fail::WriteText)
            .with_failure(Fail::Register)
            .with_reserve_failure(CF_UNICODETEXT)
            .with_content(0x77, &[(CF_UNICODETEXT, b"a\0\0\0")]);
        assert!(failing.create_window().is_err());
        assert_eq!(failing.register(Marker::Restored), None);
        failing.open(ours).expect("opens");
        assert!(matches!(failing.read(CF_UNICODETEXT, 64), Read::NoMemory));
        assert!(failing.empty().is_err());
        failing.close().expect("closes");

        let panicking = FakeClipboard::new().with_panic_at(Call::Empty);
        panicking.open(ours).expect("opens");
        assert!(catch_unwind(AssertUnwindSafe(|| panicking.empty())).is_err());
        assert_eq!(panicking.calls(), [Call::Open(0x1001), Call::Empty]);
        assert!(panicking.close().is_ok(), "usable after its panic");
    }

    /// The clipboard and global-memory calls of the `win32` block: each
    /// appears there and on no other code line of `src` or the build
    /// script. Assembled, so that this file does not match itself.
    const CALLED: [&str; 15] = [
        concat!("Open", "Clipboard"),
        concat!("Close", "Clipboard"),
        concat!("Empty", "Clipboard"),
        concat!("EnumClipboard", "Formats"),
        concat!("GetClipboard", "Data"),
        concat!("GetClipboard", "FormatName"),
        concat!("GetClipboard", "Owner"),
        concat!("GetClipboard", "SequenceNumber"),
        concat!("RegisterClipboard", "Format"),
        concat!("SetClipboard", "Data"),
        concat!("Global", "Alloc"),
        concat!("Global", "Lock"),
        concat!("Global", "Unlock"),
        concat!("Global", "Size"),
        concat!("Global", "Free"),
    ];

    /// The clipboard calls the daemon never makes, on no code line at all.
    const OTHERS: [&str; 12] = [
        concat!("CountClipboard", "Formats"),
        concat!("IsClipboard", "FormatAvailable"),
        concat!("GetOpen", "ClipboardWindow"),
        concat!("AddClipboard", "FormatListener"),
        concat!("RemoveClipboard", "FormatListener"),
        concat!("SetClipboard", "Viewer"),
        concat!("ChangeClipboard", "Chain"),
        concat!("GetPriority", "ClipboardFormat"),
        concat!("GetUpdated", "ClipboardFormats"),
        concat!("OleSet", "Clipboard"),
        concat!("OleGet", "Clipboard"),
        concat!("OleFlush", "Clipboard"),
    ];

    /// What an integration test or a doctest must not name: a call, the
    /// daemon's runner, this module, or the executor's real desk, whose
    /// keystroke would reach the machine (critique M6 of the cut).
    const PUBLIC_WAYS: [&str; 4] = [
        concat!("Win32", "Runner"),
        concat!("platform::", "clipboard"),
        concat!("Win32", "Desk"),
        concat!("WIN32", "_DESK"),
    ];

    const WIN32_BLOCK: &str = "mod win32 {";
    const CLIPBOARD_FILE: &str = "src/platform/clipboard.rs";

    /// What the scan found.
    #[derive(Default)]
    struct Findings {
        /// The calls outside the `win32` block, and the other calls anywhere,
        /// as `<file>:<line>: <code>`.
        stray: Vec<String>,
        /// The calls the `win32` block makes.
        inside: Vec<&'static str>,
        block: Option<std::ops::RangeInclusive<usize>>,
    }

    /// Scans the code lines of `text`, the source at `relative`, every line
    /// included: a test module is code that only a test build compiles.
    fn scan_source(relative: &str, text: &str, findings: &mut Findings) {
        let lines: Vec<&str> = text.lines().collect();
        let allowed = if relative == CLIPBOARD_FILE {
            findings.block = cfg_not_test_block(&lines, WIN32_BLOCK);
            findings.block.clone()
        } else {
            None
        };
        for (index, line) in lines.iter().enumerate() {
            let number = index + 1;
            let code = code_of(line);
            let calls: Vec<&'static str> = CALLED
                .into_iter()
                .filter(|call| code.contains(call))
                .collect();
            let inside = allowed
                .as_ref()
                .is_some_and(|block| block.contains(&number));
            if inside {
                findings.inside.extend(&calls);
            }
            if (!inside && !calls.is_empty()) || OTHERS.iter().any(|call| code.contains(call)) {
                findings
                    .stray
                    .push(format!("{relative}:{number}: {}", line.trim()));
            }
        }
    }

    fn names_a_way_in(line: &str) -> bool {
        CALLED
            .iter()
            .chain(&OTHERS)
            .chain(&PUBLIC_WAYS)
            .any(|token| line.contains(token))
    }

    /// The lines of the integration test `name` that name a way in, its
    /// comments included.
    fn integration_sites(name: &str, text: &str) -> Vec<String> {
        text.lines()
            .enumerate()
            .filter(|(_, line)| names_a_way_in(line))
            .map(|(index, line)| format!("{name}:{}: {}", index + 1, line.trim()))
            .collect()
    }

    /// The lines of the doctests of `relative` that name a way in.
    fn doctest_sites(relative: &str, text: &str) -> Vec<String> {
        doc_code_lines(text)
            .into_iter()
            .filter(|(_, line)| names_a_way_in(line))
            .map(|(number, line)| format!("{relative}:{number}: {}", line.trim()))
            .collect()
    }

    #[test]
    fn a_clipboard_call_in_a_tests_module_is_found() {
        // Each line is its own string literal, so the scan of this file
        // reads none of them as code.
        let text = [
            "fn production() {}",
            "#[cfg(test)]",
            "mod tests {",
            concat!("    fn probe() { unsafe { Empty", "Clipboard() }; }"),
            concat!("    fn other() { unsafe { CountClipboard", "Formats() }; }"),
            "}",
        ]
        .join("\n");
        let mut findings = Findings::default();
        scan_source("src/core/executor/paste.rs", &text, &mut findings);
        assert_eq!(findings.stray.len(), 2, "{:?}", findings.stray);
        assert!(
            findings.stray[0].starts_with("src/core/executor/paste.rs:4: "),
            "{:?}",
            findings.stray
        );
    }

    #[test]
    fn a_clipboard_token_in_a_doc_test_is_found() {
        let text = [
            "/// Pastes.",
            "///",
            "/// ```",
            concat!(
                "/// key_switch_rs::platform::clip",
                "board::SYSTEM.empty();"
            ),
            concat!("/// WIN32", "_DESK.press(&[]);"),
            "/// ```",
            concat!("/// Win32", "Runner outside a fence is prose."),
            "fn paste() {}",
        ]
        .join("\n");
        let sites = doctest_sites("src/core/executor/paste.rs", &text);
        assert_eq!(sites.len(), 2, "{sites:?}");
        assert!(
            sites[0].starts_with("src/core/executor/paste.rs:4: "),
            "{sites:?}"
        );
    }

    #[test]
    fn a_clipboard_token_in_an_integration_test_is_found() {
        let text = [
            "use key_switch_rs::ipc;",
            concat!("// a comment that names Get", "ClipboardData counts too"),
            concat!("use key_switch_rs::core::executor::Win32", "Runner;"),
            "fn main() {}",
        ]
        .join("\n");
        let sites = integration_sites("tests/scratch.rs", &text);
        assert_eq!(sites.len(), 2, "{sites:?}");
        assert!(sites[0].starts_with("tests/scratch.rs:2: "), "{sites:?}");
    }

    #[test]
    fn the_clipboard_calls_are_compiled_out_of_the_tests() {
        // Owner request 2026-10-30, ruling 9: every clipboard call sits in
        // the `win32` module of this file, which `#[cfg(not(test))]` keeps
        // out of the test build, so no test can reach one. No other line of
        // `src` or of the build script may name one, test modules included;
        // the calls the daemon never makes appear nowhere. The integration
        // tests and the doctests link the library as the daemon does, so
        // none of them may name a call, the daemon's runner, this module or
        // the executor's real desk (critique M10 of the research, M6 of the
        // cut), comments included.
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let files = scanned_sources(root);
        assert!(files.len() > 40, "the scan found the sources: {files:?}");

        let mut findings = Findings::default();
        let mut doctests = Vec::new();
        for file in &files {
            let text = std::fs::read_to_string(file).expect("read a source file");
            let relative = file
                .strip_prefix(root)
                .expect("a file under the manifest folder")
                .to_string_lossy()
                .replace('\\', "/");
            scan_source(&relative, &text, &mut findings);
            doctests.extend(doctest_sites(&relative, &text));
        }

        let mut integration = Vec::new();
        for entry in std::fs::read_dir(root.join("tests")).expect("list tests") {
            let path = entry.expect("a folder entry").path();
            let text = std::fs::read_to_string(&path).expect("read an integration test");
            integration.extend(integration_sites(&path.display().to_string(), &text));
        }

        let Findings {
            stray,
            inside,
            block,
        } = findings;
        let mut report = String::new();
        if block.is_none() {
            report +=
                &format!("{CLIPBOARD_FILE} has no `#[cfg(not(test))]` block `{WIN32_BLOCK}`\n");
        }
        for call in CALLED {
            if block.is_some() && !inside.contains(&call) {
                report += &format!(
                    "{call} is not called in the win32 block, so the scan proves nothing about it\n"
                );
            }
        }
        for (what, sites) in [
            ("clipboard calls a test build compiles", &stray),
            ("integration tests that reach the clipboard", &integration),
            ("doctests that reach the clipboard", &doctests),
        ] {
            if !sites.is_empty() {
                report += &format!("{} {what}:\n  {}\n", sites.len(), sites.join("\n  "));
            }
        }
        assert!(report.is_empty(), "{report}");
    }
}
