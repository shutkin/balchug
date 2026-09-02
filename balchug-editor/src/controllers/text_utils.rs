pub fn mark_str(txt: &str) -> Vec<(usize, &str)> {
    let mut chars = txt.char_indices().peekable();
    let mut spans = Vec::new();
    let mut is_bold = false;
    let mut span_start_index = 0;
    let mut prev_index = 0;
    let mut next_span_start_cnt = None;
    while let Some((i, c)) = chars.next() {
        if let Some(cnt) = next_span_start_cnt {
            if cnt == 1 {
                span_start_index = i;
                next_span_start_cnt = None;
            } else {
                next_span_start_cnt = Some(cnt - 1);
            }
        }
        if !is_bold {
            if c == '*' && let Some('*') = chars.peek().map(|(_, c)| *c) {
                if prev_index > span_start_index {
                    spans.push((0, &txt[span_start_index..i]));
                }
                next_span_start_cnt = Some(2);
                is_bold = true;
            }
        } else {
            if c == '*' && let Some('*') = chars.peek().map(|(_, c)| *c) {
                if prev_index > span_start_index {
                    spans.push((1, &txt[span_start_index..i]));
                }
                next_span_start_cnt = Some(2);
                is_bold = false;
            }
        }
        prev_index = i;
    }
    if prev_index > span_start_index && next_span_start_cnt.is_none() {
        let code = if is_bold {1} else {0};
        spans.push((code, &txt[span_start_index..txt.len()]));
    }
    spans
}

#[cfg(test)]
mod tst {
    use crate::controllers::text_utils::mark_str;

    #[test]
    fn test_str_spans_processing() {
        let txt = "Нормальный text. **Жирный** текст";
        let spans = mark_str(txt);
        println!("{:?}", spans);
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0], (0, "Нормальный text. "));
        assert_eq!(spans[1], (1, "Жирный"));
        assert_eq!(spans[2], (0, " текст"));
    }

    #[test]
    fn test_whole_bold() {
        let txt = "**Only bold**";
        let spans = mark_str(txt);
        println!("{:?}", spans);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0], (1, "Only bold"));
    }
}