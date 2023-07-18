use console::style;

pub fn report_error(
    file: String,
    input: String,
    err: peg::error::ParseError<peg::str::LineCol>,
) -> String {
    let mut result = String::new();
    result += format!("Failed to parse {} file!\n", file).as_str();
    let line = err.location.line;
    let col = err.location.column;
    let ep = err
        .expected
        .tokens()
        .map(|x| String::from(x))
        .reduce(|a, b| format!("{}, {}", a, b));
    result += format!("At {}:{}:{}", file, line, col).as_str();
    if let Some(ep) = ep {
        result += format!("Expected: one of {}\n", ep).as_str();
    }
    let lines = input.lines();
    let mut i = 0_usize;
    let mut pos = 0_usize;
    for line_str in lines {
        pos += 1;
        let to_sub = 3.min(line);
        if pos >= line - to_sub && pos < line + 5 {
            let n = pos.to_string();
            let numeric = format!("{:>4}", n);
            let numeric = style(numeric).blue();
            let divider = style("|").green().bold();
            result += format!("{} {} {}\n", numeric, divider, line_str).as_str();
            if line == pos {
                let ted = line_str.len() - col;
                let premark = style("     |").red().bold();
                let spanner = format!(" {:2$}{:^<3$}", " ", "^", col - 2, ted);
                let spanner = style(spanner).red();
                let here = style("here").red();
                result += format!("{} {} {here}\n", premark, spanner).as_str();
            }
            i += 1;
        }
        if i > 5 {
            break;
        }
    }
    result
}
