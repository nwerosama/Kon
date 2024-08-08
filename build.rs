fn main() {
  #[cfg(feature = "production")]
  {
    if let Ok(git_commit_hash) = std::env::var("GIT_COMMIT_HASH") {
      println!("cargo:rustc-env=GIT_COMMIT_HASH={}", &git_commit_hash[..7]);
    } else {
      println!("cargo:rustc-env=GIT_COMMIT_HASH=no_env_set");
    }
  }
}
