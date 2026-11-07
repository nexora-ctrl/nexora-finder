fn main() {
    let llvm_bin = "C:/Users/JUNIOR-JR/AppData/Local/Microsoft/WinGet/Packages/MartinStorsjo.LLVM-MinGW.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe/llvm-mingw-20261020-ucrt-x86_64/bin";
    if std::path::Path::new(llvm_bin).exists() {
        if let Ok(current_path) = std::env::var("PATH") {
            std::env::set_var("PATH", format!("{};{}", llvm_bin, current_path));
        }
        std::env::set_var("WINDRES", format!("{}/windres.exe", llvm_bin));
        std::env::set_var("RC", format!("{}/windres.exe", llvm_bin));
    }
    println!("cargo:rerun-if-changed=../dist");
    println!("cargo:rerun-if-changed=../dist/index.html");
    println!("cargo:rerun-if-changed=tauri.conf.json");
    tauri_build::build()
}
