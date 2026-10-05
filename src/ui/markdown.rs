use egui::{Color32, Frame, Margin, Rounding, Stroke, Ui};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use crate::theme::palette::{Palette, Scheme};

pub fn render(ui: &mut Ui, p: &Palette, md: &str) {
    if md.trim().is_empty() {
        ui.label(egui::RichText::new("Preview appears here…")
        .color(p.text_mute).size(12.0));
        return;
    }

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(md, opts);

    // Small state machine.
    let mut bold = false;
    let mut italic = false;
    let mut code_block = false;
    let mut code_buf = String::new();
    let mut para_buf = String::new();
    let mut para_style = ParaStyle::Plain;
    let mut list_depth: usize = 0;
    let mut list_stack: Vec<Option<u64>> = Vec::new();

    for ev in parser {
        match ev {
            Event::Start(Tag::Heading { level, .. }) => {
                flush(ui, p, &mut para_buf, &mut para_style);
                para_style = ParaStyle::Heading(level);
            }
            Event::End(TagEnd::Heading(_)) => flush(ui, p, &mut para_buf, &mut para_style),

            Event::Start(Tag::Paragraph) => {
                flush(ui, p, &mut para_buf, &mut para_style);
                para_style = ParaStyle::Plain;
            }
            Event::End(TagEnd::Paragraph) => {
                flush(ui, p, &mut para_buf, &mut para_style);
                ui.add_space(6.0);
            }

            Event::Start(Tag::Strong)        => { bold = true; }
            Event::End(TagEnd::Strong)       => { bold = false; }
            Event::Start(Tag::Emphasis)      => { italic = true; }
            Event::End(TagEnd::Emphasis)     => { italic = false; }
            Event::Start(Tag::Strikethrough) => { italic = true; }
            Event::End(TagEnd::Strikethrough)=> { italic = false; }

            Event::Start(Tag::CodeBlock(_)) => {
                flush(ui, p, &mut para_buf, &mut para_style);
                code_block = true;
                code_buf.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                code_block = false;
                Frame::new()
                .fill(Color32::from_black_alpha(if matches!(p.scheme, Scheme::Light) { 30 } else { 110 }))
                .stroke(Stroke::new(1.0, p.border))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new(code_buf.trim_end())
                        .monospace().color(p.text).size(12.0),
                    ).wrap_mode(egui::TextWrapMode::Extend));
                });
                ui.add_space(8.0);
            }
            Event::Code(t) => {
                para_buf.push('`');
                para_buf.push_str(&t);
                para_buf.push('`');
            }

            Event::Start(Tag::List(start)) => {
                flush(ui, p, &mut para_buf, &mut para_style);
                list_depth += 1;
                list_stack.push(start);
            }
            Event::End(TagEnd::List(_)) => {
                list_depth = list_depth.saturating_sub(1);
                list_stack.pop();
                if list_depth == 0 { ui.add_space(4.0); }
            }
            Event::Start(Tag::Item) => {
                flush(ui, p, &mut para_buf, &mut para_style);
                let marker = match list_stack.last_mut() {
                    Some(Some(n)) => { let s = format!("{n}. "); *n += 1; s }
                    _ => "• ".to_string(),
                };
                para_buf.push_str(&"  ".repeat(list_depth.saturating_sub(1)));
                para_buf.push_str(&marker);
            }
            Event::End(TagEnd::Item) => flush(ui, p, &mut para_buf, &mut para_style),

            Event::Start(Tag::BlockQuote(_)) => {
                flush(ui, p, &mut para_buf, &mut para_style);
                para_style = ParaStyle::Quote;
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                flush(ui, p, &mut para_buf, &mut para_style);
                para_style = ParaStyle::Plain;
            }

            Event::Rule => {
                flush(ui, p, &mut para_buf, &mut para_style);
                ui.add_space(6.0);
                let w = ui.available_width();
                let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 1.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 0.0, p.border);
                ui.add_space(6.0);
            }

            Event::Text(t) => {
                if code_block { code_buf.push_str(&t); }
                else { para_buf.push_str(&t); }
            }
            Event::SoftBreak => {
                if code_block { code_buf.push('\n'); } else { para_buf.push(' '); }
            }
            Event::HardBreak => {
                if code_block { code_buf.push('\n'); } else { para_buf.push('\n'); }
            }
            Event::Html(_) | Event::InlineHtml(_) => { /* ignore */ }
            _ => {}
        }
    }
    flush(ui, p, &mut para_buf, &mut para_style);
}

#[derive(Clone, Copy)]
enum ParaStyle { Plain, Quote, Heading(HeadingLevel) }

fn flush(ui: &mut Ui, p: &Palette, buf: &mut String, style: &mut ParaStyle) {
    let text = buf.trim();
    if text.is_empty() { *buf = String::new(); return; }

    let runs = inline_runs(text);
    let style = *style;

    match style {
        ParaStyle::Heading(level) => {
            let size = match level {
                HeadingLevel::H1 => 24.0,
                HeadingLevel::H2 => 19.5,
                HeadingLevel::H3 => 16.5,
                HeadingLevel::H4 => 15.0,
                HeadingLevel::H5 => 14.0,
                HeadingLevel::H6 => 13.0,
            };
            ui.horizontal_wrapped(|ui| {
                for r in runs {
                    let mut t = egui::RichText::new(r.text)
                    .size(size).strong().color(p.accent_1);
                    if r.mono   { t = t.monospace(); }
                    if r.italic { t = t.italics(); }
                    ui.label(t);
                }
            });
            ui.add_space(2.0);
        }
        ParaStyle::Quote => {
            Frame::new()
            .fill(Color32::TRANSPARENT)
            .inner_margin(Margin { left: 12, right: 0, top: 0, bottom: 0 })
            .show(ui, |ui| {
                let rect = ui.max_rect();
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(rect.min, egui::vec2(3.0, rect.height())),
                                         Rounding::same(2.0), p.accent_1,
                );
                ui.horizontal_wrapped(|ui| {
                    for r in runs {
                        let mut t = egui::RichText::new(r.text)
                        .size(13.5).color(p.text_dim);
                        if r.mono   { t = t.monospace(); }
                        if r.bold   { t = t.strong(); }
                        if r.italic { t = t.italics(); }
                        ui.label(t);
                    }
                });
            });
        }
        ParaStyle::Plain => {
            ui.horizontal_wrapped(|ui| {
                for r in runs {
                    let mut t = egui::RichText::new(r.text)
                    .size(14.0).color(p.text);
                    if r.mono   {
                        t = t.monospace()
                        .background_color(Color32::from_black_alpha(
                            if matches!(p.scheme, Scheme::Light) { 20 } else { 60 }));
                    }
                    if r.bold   { t = t.strong(); }
                    if r.italic { t = t.italics(); }
                    ui.label(t);
                }
            });
        }
    }
    *buf = String::new();
}

struct InlineRun { text: String, mono: bool, bold: bool, italic: bool }

fn inline_runs(s: &str) -> Vec<InlineRun> {
    let mut out: Vec<InlineRun> = Vec::new();
    let mut cur = String::new();
    let mut mono = false; let mut bold = false; let mut italic = false;
    let mut chars = s.chars().peekable();

    macro_rules! flush {
        () => {
            if !cur.is_empty() {
                out.push(InlineRun {
                    text: std::mem::take(&mut cur),
                         mono, bold, italic,
                });
            }
        };
    }

    while let Some(c) = chars.next() {
        match c {
            '`' => { flush!(); mono = !mono; }
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                    flush!();
                    bold = !bold;
                } else {
                    flush!();
                    italic = !italic;
                }
            }
            _ => cur.push(c),
        }
    }
    flush!();
    out
}
