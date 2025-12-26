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

    pub fn menu_item_aligned(description: &str, commands: &str, highlight: Option<&str>, max_len: usize) {
        let desc = if let Some(hl) = highlight {
            description.replace(hl, &format!("{}", hl.cyan().bold()))
        } else {
            description.to_string()
        };

        // Calculate padding to align all command text
        // Use the original description length (without ANSI codes) for accurate padding
        let desc_len = description.len();
        // Always ensure at least 2 spaces of padding, and pad shorter descriptions to max_len
        let padding = if desc_len < max_len {
            max_len - desc_len
        } else {
            // Even for the longest description, add minimum padding
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

    pub fn label_value_padded(label: &str, value: &str, max_label_len: usize) {
        // Same as label_value but with consistent padding to align all "::"
        let label_len = label.len();
        let padding = if label_len < max_label_len {
            max_label_len - label_len
        } else {
            0
        };
        
        println!("\t{}{}\t :: {}", label.magenta().bold(), " ".repeat(padding), value);
    }

    pub fn label_value_colored(label: &str, value: &str, label_color: &colored::ColoredString, max_label_len: usize) {
        // Same format as label_value_padded but with custom label color
        let label_len = label.len();
        let padding = if label_len < max_label_len {
            max_label_len - label_len
        } else {
            0
        };
        
        println!("\t{}{}\t :: {}", label_color, " ".repeat(padding), value.green());
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

