use std::{env, path::PathBuf, process::Command};

fn run(cmd: &mut Command) {
    let status = cmd.status().expect("could not execute native build tool");
    assert!(status.success(), "native build failed: {cmd:?}");
}

fn main() {
    let target = env::var("TARGET").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    assert!(
        target == "aarch64-linux-android" || target_os == "linux",
        "supported targets are aarch64-linux-android and native Linux"
    );
    println!("cargo:rerun-if-changed=native");
    println!("cargo:rerun-if-changed=vendor/boringssl.lock.json");
    for key in [
        "ALTDB_BORINGSSL_DIR",
        "ALTDB_NATIVE_JOBS",
        "CARGO_NDK_SYSROOT_PATH",
        "CARGO_NDK_SYSROOT_LIBS_PATH",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
    }
    if target_os == "android" && env::var_os("CARGO_NDK_SYSROOT_PATH").is_none() {
        // cargo check / IDE sync only need Rust metadata; our FFI declarations
        // are handwritten and do not require generated bindings. Keep a link
        // requirement so cargo build cannot silently produce an incomplete ELF.
        println!("cargo:rustc-link-lib=static=altdb_crypto");
        println!(
            "cargo:warning=Android metadata only; use cargo ndk -t arm64-v8a -P 30 build for native linking"
        );
        return;
    }
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let source = env::var_os("ALTDB_BORINGSSL_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("vendor/boringssl"));
    assert!(
        source.join("CMakeLists.txt").is_file(),
        "BoringSSL is missing; run python3 scripts/fetch-deps.py"
    );
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("native-cargo-ndk");
    let mut configure = Command::new("cmake");
    configure
        .arg("-S")
        .arg(root.join("native"))
        .arg("-B")
        .arg(&out)
        .args([
            "-G",
            "Ninja",
            "-DCMAKE_BUILD_TYPE=Release",
            "-DCMAKE_POSITION_INDEPENDENT_CODE=ON",
        ])
        .arg(format!(
            "-DBORINGSSL_SOURCE_DIR={}",
            source.display().to_string().replace('\\', "/")
        ));
    if target_os == "android" {
        let sysroot = PathBuf::from(
            env::var_os("CARGO_NDK_SYSROOT_PATH")
                .expect("build Android with cargo ndk -t arm64-v8a -P 30 build --locked --release"),
        );
        let libs = PathBuf::from(env::var_os("CARGO_NDK_SYSROOT_LIBS_PATH").unwrap());
        // cargo-ndk 4.1.2 exports <ndk>/toolchains/llvm/prebuilt/<host>/sysroot,
        // but not the NDK root. Let CMake select its tools from that same NDK;
        // never select a host tag or search SDK installations ourselves.
        let ndk = sysroot
            .ancestors()
            .nth(5)
            .expect("invalid cargo-ndk sysroot");
        configure
            .arg(format!(
                "-DCMAKE_ANDROID_NDK={}",
                ndk.display().to_string().replace('\\', "/")
            ))
            .args([
                "-DCMAKE_SYSTEM_NAME=Android",
                "-DCMAKE_SYSTEM_VERSION=30",
                "-DCMAKE_ANDROID_ARCH_ABI=arm64-v8a",
                "-DCMAKE_ANDROID_STL_TYPE=c++_static",
            ]);
        println!("cargo:rustc-link-arg=-Wl,-z,max-page-size=16384");
        // The API-specific directory must precede the common directory, which
        // contains libc.a alongside libc++_static.a. Otherwise -lc accidentally
        // picks static bionic while using the dynamic Android startup objects.
        println!(
            "cargo:rustc-link-search=native={}",
            libs.join("30").display()
        );
        println!("cargo:rustc-link-search=native={}", libs.display());
    }
    run(&mut configure);
    run(Command::new("cmake")
        .arg("--build")
        .arg(&out)
        .args(["--target", "altdb_crypto", "--parallel"])
        .arg(env::var("ALTDB_NATIVE_JOBS").unwrap_or_else(|_| "4".into())));
    println!(
        "cargo:rustc-link-search=native={}",
        out.join("lib").display()
    );
    for lib in ["altdb_crypto", "ssl", "crypto"] {
        println!("cargo:rustc-link-lib=static={lib}");
    }
    if target_os == "android" {
        println!("cargo:rustc-link-lib=static=c++_static");
        println!("cargo:rustc-link-lib=static=c++abi");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
    }
}
