use colored::Colorize;

pub struct Printer;

impl Printer {
    pub fn info(msg: &str) {
        println!("{}", msg.cyan());
    }

    pub fn success(msg: &str) {
        println!("{}", msg.green());
    }

    pub fn error(msg: &str) {
        println!("{}", msg.red());
    }

    pub fn warning(msg: &str) {
        println!("{}", msg.yellow());
    }

    pub fn bullet() -> String {
        format!("{}", "[*]".bold())
    }

    pub fn menu_item(description: &str, commands: &str, highlight: Option<&str>) {
        let desc = if let Some(hl) = highlight {
            description.replace(hl, &format!("{}", hl.cyan().bold()))
        } else {
            description.to_string()
        };

        // Calculate padding based on original description length (without ANSI codes)
        let desc_len = description.len();
        let padding = if desc_len < 50 {
            (50 - desc_len).max(0)
        } else {
            2
        };

        println!(
            "{} {}{} {}",
            Self::bullet(),
            desc,
            " ".repeat(padding),
            commands.yellow()
        );
    }

    pub fn header(text: &str, separator: &str) {
        if let Some(pos) = text.find(separator) {
            let (left_part, right_part) = text.split_at(pos);
            let right = if right_part.len() > separator.len() {
                &right_part[separator.len()..]
            } else {
                ""
            };
            println!(
                "{} {} {}",
                left_part.trim().cyan(),
                separator.bold(),
                right.trim()
            );
        } else {
            println!("{}", text.cyan().bold());
        }
    }

    pub fn label_value(label: &str, value: &str) {
        // Match Python: f"\t{label_color}{label}\t :: {value}{Colors.NORMAL}"
        println!("\t{}\t :: {}", label.magenta().bold(), value);
    }

    pub fn directory_item(number: u8, name: &str, path: &std::path::Path) {
        // Match Python format exactly: "{number}. {name}: {path}"
        // Python: f"{Colors.GREEN}{Colors.BOLD}{number}. {Colors.NORMAL}{Colors.RED}{Colors.BOLD}{name}:{Colors.NORMAL}{Colors.NORMAL} {path}"
        println!(
            "{} {}{}: {}",
            format!("{}.", number).green().bold(),
            name.red().bold(),
            "".normal(),
            path.display()
        );
    }
}

