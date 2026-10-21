use std::fs::OpenOptions;
use std::hash::{DefaultHasher, Hasher};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::LazyLock;
use std::{env, fs};

use super::{CHUNK, base};
use crate::lua::*;

static WORKSPACE: LazyLock<PathBuf> = LazyLock::new(|| {
    let home = PathBuf::from(env::var_os("HOME").unwrap_or_default());
    let workspace = home.join("Sunlight/workspace");
    let _ = fs::create_dir_all(&workspace);
    workspace
});

const BLOCKED: &[&str] = &[
    "exe", "com", "scr", "pif", "cpl", "msc", "bat", "cmd", "ps1", "psd1",
    "vbs", "vbe", "js", "jse", "wsf", "wsh", "hta", "scf", "lnk", "zip",
    "rar", "7z", "cab", "iso", "img", "xml", "msi", "msp", "reg", "inf",
    "url", "sh", "command", "dylib", "app",
];

pub unsafe extern "C-unwind" fn readfile(l: State) -> i32 {
    unsafe {
        let data = fs::read(resolve(l, 1)).unwrap_or_else(|e| fail(l, e));
        push_bytes(l, &data);
    }
    1
}

pub unsafe extern "C-unwind" fn writefile(l: State) -> i32 {
    unsafe {
        let path = writable(l, 1);
        let parent = path.parent().unwrap_or(&WORKSPACE);
        fs::create_dir_all(parent)
            .and_then(|()| fs::write(&path, bytes(l, 2)))
            .unwrap_or_else(|e| fail(l, e));
    }
    0
}

pub unsafe extern "C-unwind" fn appendfile(l: State) -> i32 {
    unsafe {
        OpenOptions::new()
            .append(true)
            .create(true)
            .open(writable(l, 1))
            .and_then(|mut file| file.write_all(bytes(l, 2)))
            .unwrap_or_else(|e| fail(l, e));
    }
    0
}

pub unsafe extern "C-unwind" fn isfile(l: State) -> i32 {
    unsafe { lua_pushboolean(l, resolve(l, 1).is_file() as i32) };
    1
}

pub unsafe extern "C-unwind" fn isfolder(l: State) -> i32 {
    unsafe { lua_pushboolean(l, resolve(l, 1).is_dir() as i32) };
    1
}

pub unsafe extern "C-unwind" fn makefolder(l: State) -> i32 {
    unsafe {
        fs::create_dir_all(resolve(l, 1)).unwrap_or_else(|e| fail(l, e))
    };
    0
}

pub unsafe extern "C-unwind" fn delfile(l: State) -> i32 {
    let _ = fs::remove_file(unsafe { resolve(l, 1) });
    0
}

pub unsafe extern "C-unwind" fn delfolder(l: State) -> i32 {
    let path = unsafe { resolve(l, 1) };
    if path == *WORKSPACE {
        unsafe { error(l, c"cannot delete the workspace") }
    }
    let _ = fs::remove_dir_all(path);
    0
}

pub unsafe extern "C-unwind" fn listfiles(l: State) -> i32 {
    unsafe {
        let entries =
            fs::read_dir(resolve(l, 1)).unwrap_or_else(|e| fail(l, e));
        lua_createtable(l, 0, 0);
        for (i, entry) in entries.flatten().enumerate() {
            let path = entry.path();
            let relative = path.strip_prefix(&*WORKSPACE).unwrap_or(&path);
            lua_pushinteger(l, i as i32 + 1);
            push_bytes(l, relative.as_os_str().as_encoded_bytes());
            lua_rawset(l, -3);
        }
    }
    1
}

pub unsafe extern "C-unwind" fn loadfile(l: State) -> i32 {
    unsafe {
        let source = fs::read(resolve(l, 1)).unwrap_or_else(|e| fail(l, e));
        base::chunk(l, &source, opt_string(l, 2, CHUNK.as_ptr()))
    }
}

pub unsafe extern "C-unwind" fn getcustomasset(l: State) -> i32 {
    unsafe {
        let path = resolve(l, 1);
        let data = fs::read(&path).unwrap_or_else(|e| fail(l, e));
        let mut hasher = DefaultHasher::new();
        hasher.write(&data);
        let extension = path.extension().unwrap_or_default().to_string_lossy();
        let name = format!("{:016x}.{extension}", hasher.finish());
        let content = env::current_exe()
            .unwrap_or_default()
            .with_file_name("Frameworks/RobloxLib.framework/content/sunlight");
        fs::create_dir_all(&content)
            .and_then(|()| fs::write(content.join(&name), data))
            .unwrap_or_else(|e| fail(l, e));
        push_bytes(l, format!("rbxasset://sunlight/{name}").as_bytes());
    }
    1
}

/// Maps a script path into the workspace, rejecting anything that escapes it.
unsafe fn resolve(l: State, index: i32) -> PathBuf {
    let path =
        String::from_utf8_lossy(unsafe { bytes(l, index) }).replace('\\', "/");
    let mut resolved = WORKSPACE.clone();
    for component in Path::new(&path).components() {
        match component {
            Component::Normal(part)
                if !part.as_encoded_bytes().contains(&b':') =>
            {
                resolved.push(part);
            }
            Component::ParentDir if resolved != *WORKSPACE => {
                resolved.pop();
            }
            Component::CurDir => {}
            _ => unsafe { fail(l, format!("{path} escapes the workspace")) },
        }
    }
    resolved
}

unsafe fn writable(l: State, index: i32) -> PathBuf {
    let path = unsafe { resolve(l, index) };
    let extension =
        path.extension().map(|e| e.to_string_lossy().to_lowercase());
    if extension.is_some_and(|e| BLOCKED.contains(&e.as_str())) {
        unsafe { error(l, c"file extension is blocked") }
    }
    path
}
