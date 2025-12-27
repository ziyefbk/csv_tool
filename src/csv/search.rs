//! CSV搜索和过滤模块
//! 
//! 提供全文搜索和正则表达式过滤功能

use crate::csv::CsvRecord;
use crate::error::{CsvError, Result};
use regex::{Regex, RegexBuilder};
use std::borrow::Cow;

/// 搜索模式
#[derive(Debug, Clone)]
pub enum SearchPattern {
    /// 纯文本搜索
    Text(String),
    /// 正则表达式搜索
    Regex(Regex),
}

impl SearchPattern {
    /// 创建文本搜索模式
    pub fn text(pattern: &str, case_sensitive: bool) -> Self {
        if case_sensitive {
            SearchPattern::Text(pattern.to_string())
        } else {
            SearchPattern::Text(pattern.to_lowercase())
        }
    }

    /// 创建正则表达式搜索模式
    pub fn regex(pattern: &str, case_sensitive: bool) -> Result<Self> {
        let regex = RegexBuilder::new(pattern)
            .case_insensitive(!case_sensitive)
            .build()
            .map_err(|e| CsvError::Format(format!("无效的正则表达式: {}", e)))?;
        Ok(SearchPattern::Regex(regex))
    }

    /// 检查字符串是否匹配
    pub fn is_match(&self, text: &str, case_sensitive: bool) -> bool {
        match self {
            SearchPattern::Text(pattern) => {
                if case_sensitive {
                    text.contains(pattern)
                } else {
                    text.to_lowercase().contains(pattern)
                }
            }
            SearchPattern::Regex(regex) => regex.is_match(text),
        }
    }

    /// 查找所有匹配位置
    pub fn find_matches(&self, text: &str, case_sensitive: bool) -> Vec<(usize, usize)> {
        match self {
            SearchPattern::Text(pattern) => {
                let search_text = if case_sensitive {
                    Cow::Borrowed(text)
                } else {
                    Cow::Owned(text.to_lowercase())
                };
                
                let mut matches = Vec::new();
                let mut start = 0;
                while let Some(pos) = search_text[start..].find(pattern) {
                    let abs_pos = start + pos;
                    matches.push((abs_pos, abs_pos + pattern.len()));
                    start = abs_pos + 1;
                }
                matches
            }
            SearchPattern::Regex(regex) => {
                regex.find_iter(text)
                    .map(|m| (m.start(), m.end()))
                    .collect()
            }
        }
    }
}

/// 搜索选项
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// 搜索模式
    pub pattern: SearchPattern,
    /// 目标列索引（None表示搜索所有列）
    pub columns: Option<Vec<usize>>,
    /// 大小写敏感
    pub case_sensitive: bool,
    /// 最大结果数
    pub max_results: Option<usize>,
    /// 反向匹配（显示不匹配的行）
    pub invert_match: bool,
}

impl SearchOptions {
    /// 创建新的搜索选项
    pub fn new(pattern: SearchPattern) -> Self {
        Self {
            pattern,
            columns: None,
            case_sensitive: true,
            max_results: None,
            invert_match: false,
        }
    }

    /// 设置目标列
    pub fn with_columns(mut self, columns: Vec<usize>) -> Self {
        self.columns = Some(columns);
        self
    }

    /// 设置大小写敏感性
    pub fn with_case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    /// 设置最大结果数
    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = Some(max);
        self
    }

    /// 设置反向匹配
    pub fn with_invert_match(mut self, invert: bool) -> Self {
        self.invert_match = invert;
        self
    }
}

/// 单个匹配信息
#[derive(Debug, Clone)]
pub struct MatchInfo {
    /// 匹配的列索引
    pub column: usize,
    /// 匹配位置（起始，结束）
    pub positions: Vec<(usize, usize)>,
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// 匹配的行号（从0开始，不含表头）
    pub row_number: usize,
    /// 匹配信息列表
    pub matches: Vec<MatchInfo>,
    /// 行数据
    pub record: CsvRecord<'static>,
}

impl SearchResult {
    /// 检查指定列是否有匹配
    pub fn has_match_in_column(&self, col: usize) -> bool {
        self.matches.iter().any(|m| m.column == col)
    }

    /// 获取所有匹配的列索引
    pub fn matched_columns(&self) -> Vec<usize> {
        self.matches.iter().map(|m| m.column).collect()
    }
}

/// 搜索器
pub struct Searcher {
    options: SearchOptions,
}

impl Searcher {
    /// 创建新的搜索器
    pub fn new(options: SearchOptions) -> Self {
        Self { options }
    }

    /// 检查记录是否匹配
    pub fn matches_record(&self, record: &CsvRecord) -> Option<Vec<MatchInfo>> {
        let mut all_matches = Vec::new();

        // 确定要搜索的列
        let columns: Vec<usize> = match &self.options.columns {
            Some(cols) => cols.clone(),
            None => (0..record.fields.len()).collect(),
        };

        for &col in &columns {
            if let Some(field) = record.fields.get(col) {
                let text = field.as_ref();
                let positions = self.options.pattern.find_matches(text, self.options.case_sensitive);
                
                if !positions.is_empty() {
                    all_matches.push(MatchInfo {
                        column: col,
                        positions,
                    });
                }
            }
        }

        // 处理反向匹配
        if self.options.invert_match {
            if all_matches.is_empty() {
                // 反向匹配：没有匹配才返回
                Some(Vec::new())
            } else {
                None
            }
        } else {
            // 正常匹配
            if all_matches.is_empty() {
                None
            } else {
                Some(all_matches)
            }
        }
    }

    /// 检查记录是否简单匹配（不返回详细位置）
    pub fn is_match(&self, record: &CsvRecord) -> bool {
        let columns: Vec<usize> = match &self.options.columns {
            Some(cols) => cols.clone(),
            None => (0..record.fields.len()).collect(),
        };

        let has_match = columns.iter().any(|&col| {
            record.fields.get(col).map_or(false, |field| {
                self.options.pattern.is_match(field.as_ref(), self.options.case_sensitive)
            })
        });

        if self.options.invert_match {
            !has_match
        } else {
            has_match
        }
    }
}

/// 用于高亮显示的辅助函数
pub fn highlight_matches(text: &str, positions: &[(usize, usize)]) -> String {
    if positions.is_empty() {
        return text.to_string();
    }

    let mut result = String::new();
    let mut last_end = 0;

    for &(start, end) in positions {
        // 添加匹配前的文本
        if start > last_end {
            result.push_str(&text[last_end..start]);
        }
        // 添加高亮的匹配文本（使用 ANSI 颜色代码）
        result.push_str("\x1b[1;33m"); // 黄色加粗
        result.push_str(&text[start..end]);
        result.push_str("\x1b[0m"); // 重置
        last_end = end;
    }

    // 添加最后一段文本
    if last_end < text.len() {
        result.push_str(&text[last_end..]);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_search() {
        let pattern = SearchPattern::text("hello", true);
        assert!(pattern.is_match("hello world", true));
        assert!(!pattern.is_match("HELLO world", true));
    }

    #[test]
    fn test_text_search_case_insensitive() {
        let pattern = SearchPattern::text("hello", false);
        assert!(pattern.is_match("HELLO world", false));
        assert!(pattern.is_match("Hello World", false));
    }

    #[test]
    fn test_regex_search() {
        let pattern = SearchPattern::regex(r"\d+", true).unwrap();
        assert!(pattern.is_match("abc123def", true));
        assert!(!pattern.is_match("abcdef", true));
    }

    #[test]
    fn test_find_matches() {
        let pattern = SearchPattern::text("test", true);
        let matches = pattern.find_matches("test1 test2 test3", true);
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0], (0, 4));
        assert_eq!(matches[1], (6, 10));
        assert_eq!(matches[2], (12, 16));
    }

    #[test]
    fn test_highlight_matches() {
        let text = "hello world";
        let positions = vec![(0, 5)];
        let highlighted = highlight_matches(text, &positions);
        assert!(highlighted.contains("\x1b[1;33m"));
        assert!(highlighted.contains("hello"));
    }
}


