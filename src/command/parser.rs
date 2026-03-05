//! 命令解析器

/// 解析后的命令
#[derive(Debug, Clone)]
pub struct ParsedCommand<'a> {
    /// 主命令
    pub cmd: &'a str,
    /// 参数列表（第一个参数可作为子命令）
    pub args: Vec<&'a str>,
    /// 原始消息
    pub raw_msg: &'a str,
}

/// 命令解析器
#[derive(Clone)]
pub struct Parser {
    /// 命令前缀
    prefix: String,
}

impl Parser {
    /// 创建新的解析器
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }

    /// 获取命令前缀
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// 更新命令前缀
    pub fn set_prefix(&mut self, prefix: String) {
        self.prefix = prefix;
    }

    /// 解析消息，提取命令
    pub fn parse<'a>(&self, msg: &'a str) -> Option<ParsedCommand<'a>> {
        let msg = msg.trim();

        // 检查前缀
        if !msg.starts_with(&self.prefix) {
            return None;
        }

        // 移除前缀
        let remainder = msg[self.prefix.len()..].trim_start();

        // 空消息
        if remainder.is_empty() {
            return None;
        }

        // 解析命令和参数
        let tokens = Self::tokenize(remainder);

        if tokens.is_empty() {
            return None;
        }

        let cmd = tokens[0];
        let args = if tokens.len() > 1 {
            tokens[1..].to_vec()
        } else {
            Vec::new()
        };

        Some(ParsedCommand {
            cmd,
            args,
            raw_msg: msg,
        })
    }

    /// 分词，支持引号包裹的参数
    fn tokenize(input: &str) -> Vec<&str> {
        let mut tokens = Vec::new();
        let mut i = 0;
        let bytes = input.as_bytes();
        let len = bytes.len();

        while i < len {
            // 跳过空白字符
            if bytes[i].is_ascii_whitespace() {
                i += 1;
                continue;
            }

            // 处理引号包裹的内容
            if bytes[i] == b'"' || bytes[i] == b'\'' {
                let quote = bytes[i];
                i += 1; // 跳过开始引号
                let content_start = i;

                // 查找结束引号
                while i < len && bytes[i] != quote {
                    i += 1;
                }

                if i < len {
                    // 找到结束引号
                    tokens.push(&input[content_start..i]);
                    i += 1; // 跳过结束引号
                } else {
                    // 没有结束引号，返回剩余内容
                    tokens.push(&input[content_start..]);
                }
            } else {
                // 普通单词，找到下一个空白字符
                let start = i;
                while i < len && !bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                tokens.push(&input[start..i]);
            }
        }

        tokens
    }

    /// 检查消息是否以命令前缀开头
    pub fn is_command(&self, msg: &str) -> bool {
        msg.trim().starts_with(&self.prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let parser = Parser::new("!".to_string());
        let result = parser.parse("!help").unwrap();

        assert_eq!(result.cmd, "help");
        assert!(result.args.is_empty());
    }

    #[test]
    fn test_parse_command_with_args() {
        let parser = Parser::new("!".to_string());
        let result = parser.parse("!bot name 新名称").unwrap();

        assert_eq!(result.cmd, "bot");
        assert_eq!(result.args, vec!["name", "新名称"]);
    }

    #[test]
    fn test_parse_command_with_quoted_args() {
        let parser = Parser::new("!".to_string());
        let result = parser.parse("!meme top \"上方文字\" \"下方文字\"").unwrap();

        assert_eq!(result.cmd, "meme");
        assert_eq!(result.args, vec!["top", "上方文字", "下方文字"]);
    }

    #[test]
    fn test_parse_single_quotes() {
        let parser = Parser::new("!".to_string());
        let result = parser.parse("!cmd 'hello world'").unwrap();

        assert_eq!(result.cmd, "cmd");
        assert_eq!(result.args, vec!["hello world"]);
    }

    #[test]
    fn test_parse_no_prefix() {
        let parser = Parser::new("!".to_string());
        assert!(parser.parse("help").is_none());
    }

    #[test]
    fn test_parse_empty_after_prefix() {
        let parser = Parser::new("!".to_string());
        assert!(parser.parse("!   ").is_none());
    }

    #[test]
    fn test_parse_multiple_args() {
        let parser = Parser::new("!".to_string());
        let result = parser.parse("!cmd arg1 arg2 arg3").unwrap();

        assert_eq!(result.cmd, "cmd");
        assert_eq!(result.args, vec!["arg1", "arg2", "arg3"]);
    }

    #[test]
    fn test_tokenizer() {
        let tokens = Parser::tokenize("cmd \"arg with space\" simple");
        assert_eq!(tokens, vec!["cmd", "arg with space", "simple"]);
    }

    #[test]
    fn test_custom_prefix() {
        let parser = Parser::new("!ai ".to_string());
        let result = parser.parse("!ai hello").unwrap();

        assert_eq!(result.cmd, "hello");
    }

    #[test]
    fn test_is_command() {
        let parser = Parser::new("!".to_string());

        assert!(parser.is_command("!help"));
        assert!(parser.is_command("  !help  "));
        assert!(!parser.is_command("help"));
    }
}