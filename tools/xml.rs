//! Just enough of Python's xml.etree.ElementTree for the cut files these tools
//! write and read back: elements with attributes in order, text and tail kept,
//! and tostring() writing an element out as ET did (the svg namespace declared
//! on the element written, empty elements closed with " />").
pub struct El {
    pub tag: String,
    pub attrs: Vec<(String, String)>,
    pub kids: Vec<El>,
    pub text: String,
    pub tail: String,
}

impl El {
    pub fn get(&self, k: &str) -> Option<&str> {
        self.attrs.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str())
    }
    /// the tag without any prefix
    pub fn local(&self) -> &str {
        self.tag.rsplit(':').next().unwrap_or("")
    }
    /// ET.tostring(el, encoding='unicode'), tail included
    pub fn tostring(&self) -> String {
        let mut o = String::new();
        self.write(&mut o, true);
        o
    }
    fn write(&self, o: &mut String, top: bool) {
        o.push('<');
        o.push_str(self.local());
        if top {
            o.push_str(" xmlns=\"http://www.w3.org/2000/svg\"");
        }
        for (k, v) in &self.attrs {
            if k == "xmlns" {
                continue;
            }
            o.push_str(&format!(" {k}=\"{}\"", esc_attr(v)));
        }
        if self.kids.is_empty() && self.text.is_empty() {
            o.push_str(" />");
        } else {
            o.push('>');
            o.push_str(&esc_text(&self.text));
            for k in &self.kids {
                k.write(o, false);
            }
            o.push_str(&format!("</{}>", self.local()));
        }
        o.push_str(&esc_text(&self.tail));
    }
}

fn esc_text(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn esc_attr(s: &str) -> String {
    esc_text(s).replace('"', "&quot;").replace('\r', "&#13;").replace('\n', "&#10;").replace('\t', "&#09;")
}

fn unent(s: &str) -> String {
    let mut o = String::new();
    let mut rest = s;
    while let Some(k) = rest.find('&') {
        o.push_str(&rest[..k]);
        let r = &rest[k..];
        let Some(e) = r.find(';') else {
            o.push_str(r);
            return o;
        };
        let name = &r[1..e];
        let ch = match name {
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "amp" => Some('&'),
            _ if name.starts_with("#x") => u32::from_str_radix(&name[2..], 16).ok().and_then(char::from_u32),
            _ if name.starts_with('#') => name[1..].parse().ok().and_then(char::from_u32),
            _ => None,
        };
        match ch {
            Some(c) => o.push(c),
            None => o.push_str(&r[..=e]),
        }
        rest = &r[e + 1..];
    }
    o.push_str(rest);
    o
}

/// The root element of an XML document.
pub fn parse(src: &str) -> Result<El, String> {
    let new = |tag: String, attrs| El { tag, attrs, kids: vec![], text: String::new(), tail: String::new() };
    let mut stack: Vec<El> = vec![new(String::new(), vec![])];
    let b = src.as_bytes();
    let mut i = 0;
    // text runs go to the open element's text, or the last closed child's tail
    let put = |stack: &mut Vec<El>, t: &str| {
        let top = stack.last_mut().unwrap();
        match top.kids.last_mut() {
            Some(k) => k.tail.push_str(t),
            None => top.text.push_str(t),
        }
    };
    while i < b.len() {
        if b[i] != b'<' {
            let j = src[i..].find('<').map(|k| i + k).unwrap_or(b.len());
            if stack.len() > 1 {
                put(&mut stack, &unent(&src[i..j]));
            }
            i = j;
            continue;
        }
        for (open, close) in [("<?", "?>"), ("<!--", "-->")] {
            if src[i..].starts_with(open) {
                i = src[i..].find(close).map(|k| i + k + close.len()).ok_or("unterminated markup")?;
            }
        }
        if i >= b.len() || b[i] != b'<' {
            continue;
        }
        if src[i..].starts_with("<!") {
            i = src[i..].find('>').map(|k| i + k + 1).ok_or("unterminated <!")?;
            continue;
        }
        let mut j = i + 1;
        let mut q: Option<u8> = None;
        while j < b.len() {
            match q {
                Some(c) if b[j] == c => q = None,
                Some(_) => {}
                None if b[j] == b'"' || b[j] == b'\'' => q = Some(b[j]),
                None if b[j] == b'>' => break,
                None => {}
            }
            j += 1;
        }
        let body = &src[i + 1..j];
        i = j + 1;
        if let Some(name) = body.strip_prefix('/') {
            let el = stack.pop().ok_or("unbalanced")?;
            if el.tag != name.trim() {
                return Err(format!("mismatched tag: {} closed by {}", el.tag, name.trim()));
            }
            stack.last_mut().ok_or("unbalanced")?.kids.push(el);
            continue;
        }
        let selfclose = body.ends_with('/');
        let body = body.strip_suffix('/').unwrap_or(body);
        let tag: String = body.split(|c: char| c.is_whitespace()).next().unwrap_or("").to_string();
        let mut attrs = vec![];
        let rest = &body[tag.len()..];
        let rb = rest.as_bytes();
        let mut k = 0;
        loop {
            while k < rb.len() && rb[k].is_ascii_whitespace() {
                k += 1;
            }
            let ks = k;
            while k < rb.len() && rb[k] != b'=' && !rb[k].is_ascii_whitespace() {
                k += 1;
            }
            if k >= rb.len() {
                break;
            }
            let key = rest[ks..k].to_string();
            while k < rb.len() && rb[k] != b'"' && rb[k] != b'\'' {
                k += 1;
            }
            if k >= rb.len() {
                break;
            }
            let qc = rb[k];
            let vs = k + 1;
            k = vs;
            while k < rb.len() && rb[k] != qc {
                k += 1;
            }
            // an XML parser hands back attribute whitespace as spaces
            let v = rest[vs..k].replace(['\n', '\t', '\r'], " ");
            attrs.push((key, unent(&v)));
            k += 1;
        }
        let el = new(tag, attrs);
        if selfclose {
            stack.last_mut().ok_or("unbalanced")?.kids.push(el);
        } else {
            stack.push(el);
        }
    }
    if stack.len() != 1 {
        return Err("no element found".into());
    }
    stack.pop().unwrap().kids.pop().ok_or_else(|| "no element found".to_string())
}
