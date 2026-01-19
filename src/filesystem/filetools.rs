use indicatif::{ProgressBar, ProgressStyle};
use jwalk::WalkDir;
use rayon::prelude::*;
use std::fs;
use std::path::Path;

#[cfg(target_os = "windows")]
use windows_sys::Win32::Storage::FileSystem::{
    CopyFile2, COPYFILE2_EXTENDED_PARAMETERS, COPY_FILE_NO_BUFFERING,
};

pub fn remove(path: String, _output: String, debug: bool, bar: bool) {
    let path = Path::new(&path);
    if !path.exists() {
        return;
    }

    if path.is_file() {
        let _ = fs::remove_file(path);
        return;
    }

    let items: Vec<_> = WalkDir::new(path)
        .parallelism(jwalk::Parallelism::RayonDefaultPool {
            busy_timeout: std::time::Duration::from_secs(1),
        })
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    let pb = if bar {
        let p = ProgressBar::new(items.len() as u64);
        p.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .unwrap(),
        );
        Some(p)
    } else {
        None
    };

    items.par_iter().for_each(|entry| {
        if entry.file_type().is_file() {
            if debug {
                println!("Trash: {}", entry.path().display());
            }
            let _ = fs::remove_file(entry.path());
        }
        if let Some(ref p) = pb {
            p.inc(1);
        }
    });

    let _ = fs::remove_dir_all(path);
    if let Some(p) = pb {
        p.finish_and_clear();
    }
}

pub fn copy(src: String, dst: String, debug: bool, bar: bool) {
    let src_path = Path::new(&src);
    let dst_path = Path::new(&dst);

    if !src_path.exists() {
        return;
    }

    if src_path.is_file() {
        let _ = kernel_copy(src_path, dst_path, debug);
    } else {
        let _ = fs::create_dir_all(dst_path);
        let items: Vec<_> = WalkDir::new(src_path)
            .parallelism(jwalk::Parallelism::RayonDefaultPool {
                busy_timeout: std::time::Duration::from_secs(1),
            })
            .into_iter()
            .filter_map(|e| e.ok())
            .collect();

        let pb = if bar {
            let p = ProgressBar::new(items.len() as u64);
            p.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap());
            Some(p)
        } else {
            None
        };

        items.par_iter().for_each(|entry| {
            let path = entry.path();
            let rel = path.strip_prefix(src_path).unwrap();
            let target = dst_path.join(rel);

            if entry.file_type().is_dir() {
                let _ = fs::create_dir_all(&target);
            } else {
                if let Some(ref p) = pb {
                    p.set_message(format!("{:?}", path.file_name().unwrap_or_default()));
                }
                let _ = kernel_copy(&path, &target, debug);
            }
            if let Some(ref p) = pb {
                p.inc(1);
            }
        });
        if let Some(p) = pb {
            p.finish_and_clear();
        }
    }
}

pub fn move_file(src: String, dst: String, debug: bool, bar: bool) {
    if fs::rename(&src, &dst).is_ok() {
        return;
    }
    copy(src.clone(), dst, debug, bar);
    remove(src, String::new(), debug, bar);
}

fn kernel_copy(src: &Path, dst: &Path, debug: bool) -> Result<(), std::io::Error> {
    if debug {
        println!("{} -> {}", src.display(), dst.display());
    }

    if let Some(p) = dst.parent() {
        let _ = fs::create_dir_all(p);
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::ffi::OsStrExt;
        let s: Vec<u16> = src.as_os_str().encode_wide().chain(Some(0)).collect();
        let d: Vec<u16> = dst.as_os_str().encode_wide().chain(Some(0)).collect();

        let params = COPYFILE2_EXTENDED_PARAMETERS {
            dwSize: std::mem::size_of::<COPYFILE2_EXTENDED_PARAMETERS>() as u32,
            dwCopyFlags: COPY_FILE_NO_BUFFERING,
            pfCancel: std::ptr::null_mut(),
            pProgressRoutine: None,
            pvCallbackContext: std::ptr::null_mut(),
        };

        unsafe {
            if CopyFile2(s.as_ptr(), d.as_ptr(), &params) == 0 {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        fs::copy(src, dst).map(|_| ())
    }
}
