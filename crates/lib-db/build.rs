fn main() {
    // 解决 migrations 内部内容更新不重新打包的问题
    println!("cargo:rerun-if-changed=../../migrations");
}