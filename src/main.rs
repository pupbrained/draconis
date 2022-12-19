#![feature(let_chains)]
#![feature(local_key_cell_methods)]

use {
  colored::Colorize,
  dirs::home_dir,
  std::{
    cell::RefCell,
    fs::File,
    io::{BufRead, BufReader, Lines, Result},
    iter::Peekable,
    os::unix::process::parent_id,
    path::Path,
  },
  sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt},
  unicode_width::UnicodeWidthStr,
  users::get_current_username,
};

thread_local! {
  static LOGO: RefCell<Peekable<Lines<BufReader<File>>>> = RefCell::new(
    read_lines(
      format!("{}/.config/draconis/logo", home_dir().unwrap().to_string_lossy())
    )
      .unwrap()
      .peekable()
  );

  static LEN: RefCell<usize> = RefCell::new(
    LOGO.with_borrow_mut(|f| {
      f
        .peek()
        .unwrap()
        .as_ref()
        .unwrap()
        .to_owned()
        .replace("$C", "")
        .replace("$B", "")
        .as_str()
        .width()
    })
  );
}

#[tokio::main]
async fn main() {
  let mut sys = System::new_all();
  sys.refresh_all();

  if
        let Some(name) = sys.name() &&
        let Some(kernel) = sys.kernel_version() &&
        let Some(user) = get_current_username() &&
        let Some(process) = sys.process(Pid::from_u32(parent_id()))
    {
        println!("{} {} {}", get_logo_line(), "╭─".green(), name);
        println!("{} {} {}", get_logo_line(), "├─".green(), kernel);
        println!(
            "{} {} {}",
            get_logo_line(),
            "├─".green(),
            user.to_string_lossy()
        );
        println!("{} {} {}", get_logo_line(), "╰─".green(), process.name());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
        println!("{} :", get_logo_line());
    }

  LOGO.with_borrow_mut(|f| {
    for line in f {
      let mut line = line.unwrap();

      if line.contains("$C") {
        line = line.replace("$C", "\x1b[36m")
      }

      if line.contains("$B") {
        line = line.replace("$B", "\x1b[34m")
      }

      println!("{line}");
    }
  });
}

fn read_lines<P>(filename: P) -> Result<Lines<BufReader<File>>>
where
  P: AsRef<Path>,
{
  let file = File::open(filename)?;
  Ok(BufReader::new(file).lines())
}

fn get_logo_line() -> String {
  let len = LEN.with_borrow_mut(|f| f.clone());
  LOGO.with_borrow_mut(|f| {
    if let Some(line) = f.next() && let Ok(mut line) = line {
      if line.contains("$C") {
        line = line.replace("$C", "\x1b[36m")
      }

      if line.contains("$B") {
        line = line.replace("$B", "\x1b[34m")
      }

      line
    } else {
      " ".repeat(len)
    }
  })
}
