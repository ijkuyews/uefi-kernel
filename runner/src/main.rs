use std::{env::args, path::PathBuf};

use structopt::StructOpt;

#[derive(StructOpt)]
pub enum RunnerCommand {
  Build,
  Run {
    #[structopt(long)]
    debug: bool,
  },
}

fn build(kernel_path: &PathBuf) -> anyhow::Result<PathBuf> {
  println!("Kernel binary is at {}", kernel_path.display());

  let bootloader_manifest = bootloader_locator::locate_bootloader("bootloader")?;
  let kernel_manifest = locate_cargo_manifest::locate_manifest()?;

  let _dir = xshell::pushd(bootloader_manifest.parent().unwrap())?;
  let image_target = kernel_manifest.parent().unwrap().join("target");
  let image_out = kernel_path.parent().unwrap();

  xshell::cmd!(
    "
			cargo builder
				--firmware uefi
				--kernel-manifest {kernel_manifest}
				--kernel-binary {kernel_path}
				--target-dir {image_target}
				--out-dir {image_out}
		"
  )
  .run()?;

  let kernel_binary_name = kernel_path.file_name().unwrap().to_str().unwrap();
  let disk_image = kernel_path.parent().unwrap().join(format!("boot-uefi-{}.img", kernel_binary_name));

  if !disk_image.exists() {
    panic!("Disk image does not exist at {} after bootloader build", disk_image.display());
  }

  Ok(disk_image)
}

fn build_and_run(kernel_path: &PathBuf, debug: bool) -> anyhow::Result<()> {
  let disk_image = build(kernel_path)?;

  let disk_image_arg = disk_image.display().to_string();
  let additional_args: &[&str] = if debug { &["-s", "-S"] } else { &[] };

  xshell::cmd!(
    "
		qemu-system-x86_64
		  -nodefaults
			-drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd
			-drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.fd
			-drive format=raw,file={disk_image_arg}
			-machine type=q35
			-m 256M
			-smp 4
			-vga std
			{additional_args...}
		"
  )
  .run()?;

  Ok(())
}

fn main() -> anyhow::Result<()> {
  let mut args = args().collect::<Vec<_>>();

  let kernel_file = PathBuf::from(args.remove(1)).canonicalize()?;
  let command = RunnerCommand::from_iter(args);

  match command {
    RunnerCommand::Build => {
      build(&kernel_file)?;
    }
    RunnerCommand::Run { debug } => build_and_run(&kernel_file, debug)?,
  }

  Ok(())
}
