use std::{ffi::OsStr, fs, io::Write, os::unix::ffi::OsStrExt, path::PathBuf, process::Command};

use anyhow::Result;

use crate::{config::ProjectConfig, os::new_config, shell::Shell};

pub struct Project {
    workdir: PathBuf,
    pub config: ProjectConfig,
    pub arch: Arch,
    pub bin_path: Option<PathBuf>,
}

impl Project {
    pub fn new(workdir: PathBuf, config: Option<String>) -> Result<Self> {
        let config_path = config
            .map(PathBuf::from)
            .unwrap_or(workdir.join(".project.toml"));
        let config;
        if !fs::exists(&config_path)? {
            config = new_config(&workdir);
            let config_str = toml::to_string(&config).unwrap();
            let mut file = fs::File::create(&config_path).unwrap();
            file.write_all(config_str.as_bytes()).unwrap();
        } else {
            config = toml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
        }
        let arch = Arch::from_target(&config.compile.target).unwrap();

        Ok(Self {
            workdir,
            config,
            bin_path: None,
            arch,
        })
    }

    pub fn shell<S: AsRef<OsStr>>(&self, program: S) -> Command {
        let mut cmd = Command::new(program);
        cmd.current_dir(&self.workdir);
        cmd
    }

    pub fn install_deps(&self) {
        self.shell("cargo")
            .args(["install", "cargo-binutils"])
            .exec()
            .unwrap();
        self.shell("rustup")
            .args(["component", "add", "llvm-tools-preview", "rust-src"])
            .exec()
            .unwrap();
    }

    pub fn output_dir(&self, debug: bool) -> PathBuf {
        let pwd = self.workdir.clone();

        let target = &self.config.compile.target;

        pwd.join("target")
            .join(target)
            .join(if debug { "debug" } else { "release" })
    }

    pub fn package_metadata(&self) -> serde_json::Value {
        let meta = self.cargo_meta();
        let packages = meta["packages"].as_array().unwrap();
        let package = packages
            .iter()
            .find(|one| one["name"] == self.config.compile.package)
            .unwrap();

        package.clone()
    }

    pub fn package_dependencies(&self) -> Vec<String> {
        let meta = self.package_metadata();

        meta["dependencies"]
            .as_array()
            .unwrap()
            .iter()
            .map(|one| one["name"].as_str().unwrap().to_string())
            .collect()
    }

    fn cargo_meta(&self) -> serde_json::Value {
        let output = Command::new("cargo")
            .current_dir(&self.workdir)
            .args(["metadata", "--format-version=1", "--no-deps"])
            .output()
            .unwrap();
        let stdout = OsStr::from_bytes(&output.stdout);
        let data = stdout.to_str().unwrap();

        serde_json::from_str(data).unwrap()
    }
}

pub enum Arch {
    Aarch64,
    Riscv64,
    X86_64,
}

impl Default for Arch {
    fn default() -> Self {
        Self::Aarch64
    }
}

impl Arch {
    pub fn qemu_arch(&self) -> String {
        let arch = match self {
            Arch::Aarch64 => "aarch64",
            Arch::Riscv64 => "riscv64",
            Arch::X86_64 => "x86_64",
        };

        format!("qemu-system-{}", arch)
    }

    fn from_target(target: &str) -> Result<Arch> {
        if target.contains("aarch64") {
            return Ok(Arch::Aarch64);
        }

        if target.contains("riscv64") {
            return Ok(Arch::Riscv64);
        }

        if target.contains("x86_64") {
            return Ok(Arch::X86_64);
        }

        Err(anyhow::anyhow!("Unsupportedtarget: {}", target))
    }
}
