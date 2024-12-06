fn main() {
  #[cfg(feature = "production")]
  {
    if let Ok(git_commit_hash) = std::env::var("GIT_COMMIT_HASH") {
      println!("cargo:rustc-env=GIT_COMMIT_HASH={}", &git_commit_hash[..7]);
    } else {
      println!("cargo:warning=GIT_COMMIT_HASH not found");
      println!("cargo:rustc-env=GIT_COMMIT_HASH=no_env_set");
    }
  }

  {
    let git_branch = std::process::Command::new("git")
      .args(["rev-parse", "--abbrev-ref", "HEAD"])
      .output()
      .expect("Command execution failed");

    if git_branch.status.success() {
      let git_branch = String::from_utf8(git_branch.stdout).expect("Invalid UTF-8 sequence").trim().to_string();
      println!("cargo:rustc-env=GIT_COMMIT_BRANCH={}", &git_branch);
    } else {
      println!("cargo:warning=GIT_COMMIT_BRANCH not found");
      println!("cargo:rustc-env=GIT_COMMIT_BRANCH=no_env_set");
    }
  }
}
