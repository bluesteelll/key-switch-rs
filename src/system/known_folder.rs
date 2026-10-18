//! Known folders of the account the daemon runs as.

use std::ffi::{OsString, c_void};
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

use windows::{
    Win32::{
        System::Com::CoTaskMemFree,
        UI::Shell::{FOLDERID_LocalAppData, KF_FLAG_DEFAULT, SHGetKnownFolderPath},
    },
    core::PWSTR,
};

/// The `%LOCALAPPDATA%` folder of the account this process runs as, from
/// the shell rather than the environment, which whoever started the
/// elevated daemon may have changed or left without it.
pub fn local_app_data() -> windows::core::Result<PathBuf> {
    // SAFETY: FOLDERID_LocalAppData is a static GUID, so the pointer stays
    // valid for the call; KF_FLAG_DEFAULT and no token (the current user)
    // are documented arguments. The wrapper owns the out-pointer and hands
    // back the string the shell allocated, which `TaskMemString` frees.
    let path = TaskMemString(unsafe {
        SHGetKnownFolderPath(&FOLDERID_LocalAppData, KF_FLAG_DEFAULT, None)
    }?);
    // SAFETY: on success the PWSTR points to a NUL-terminated string that
    // `path` owns and frees only when dropped, after this copy.
    let wide = unsafe { path.0.as_wide() };
    Ok(PathBuf::from(OsString::from_wide(wide)))
}

/// A string the shell allocated with `CoTaskMemAlloc`, freed on drop.
struct TaskMemString(PWSTR);

impl Drop for TaskMemString {
    fn drop(&mut self) {
        // SAFETY: the pointer came from a successful SHGetKnownFolderPath,
        // which allocates it with CoTaskMemAlloc for the caller to free;
        // this guard is its only owner, so it is freed once.
        unsafe { CoTaskMemFree(Some(self.0.0.cast_const().cast::<c_void>())) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A read of the shell's folder, on a test thread that initialised no
    /// COM: neither call needs it.
    #[test]
    fn local_app_data_is_an_existing_absolute_folder() {
        let folder = local_app_data().expect("the LocalAppData folder is known");
        assert!(folder.is_absolute(), "{}", folder.display());
        assert!(folder.is_dir(), "{}", folder.display());
    }
}
