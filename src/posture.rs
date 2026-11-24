pub enum Direction { Added, Removed }

pub struct Cost {
    pub dimension: &'static str,
    pub value: i32,
    pub desc: &'static str,
}

impl Cost {
    /// Map a security assertion key to its cost (or refund).
    pub fn of(key: &str, direction: Direction) -> Option<Cost> {
        use Direction::*;
        let c = match (key, direction) {
            // Auth decorators
            (k, Removed) if k.starts_with("auth:") =>
                Cost { dimension: "auth_coverage", value: -15, desc: "Login protection removed" },
            (k, Added) if k.starts_with("auth:") =>
                Cost { dimension: "auth_coverage", value: 15, desc: "Login protection added" },

            // Dangerous call additions
            ("call:subprocess:shell=True", Added) =>
                Cost { dimension: "input_validation", value: -30, desc: "Shell injection surface added" },
            ("call:subprocess:shell=True", Removed) =>
                Cost { dimension: "input_validation", value: 30, desc: "Shell injection fixed" },

            ("call:requests:verify=False", Added) =>
                Cost { dimension: "crypto_strength", value: -25, desc: "TLS verification disabled" },
            ("call:requests:verify=False", Removed) =>
                Cost { dimension: "crypto_strength", value: 25, desc: "TLS verification restored" },

            ("call:eval", Added) =>
                Cost { dimension: "input_validation", value: -30, desc: "eval() added" },
            ("call:exec", Added) =>
                Cost { dimension: "input_validation", value: -30, desc: "exec() added" },

            // Config changes
            ("config:DEBUG=True", Added) =>
                Cost { dimension: "secret_exposure", value: -20, desc: "DEBUG mode enabled" },
            ("config:DEBUG=True", Removed) =>
                Cost { dimension: "secret_exposure", value: 20, desc: "DEBUG mode disabled" },

            ("config:CORS=*", Added) =>
                Cost { dimension: "permission_scope", value: -25, desc: "CORS allows any origin" },
            ("config:CORS=*", Removed) =>
                Cost { dimension: "permission_scope", value: 25, desc: "CORS locked down" },

            // Dangerous imports
            ("import:pickle", Added) =>
                Cost { dimension: "crypto_strength", value: -15, desc: "Unsafe pickle import" },
            ("import:marshal", Added) =>
                Cost { dimension: "crypto_strength", value: -15, desc: "Unsafe marshal import" },

            _ => return None,
        };
        Some(c)
    }
}

/// Extract security-relevant assertions as (key, line) pairs from Python source.
pub fn assertion_set(source: &str) -> Vec<(String, usize)> {
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(tree_sitter_python::language()).is_err() {
        return vec![];
    }
    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return vec![],
    };
    let mut out = Vec::new();
    visit(tree.root_node(), source.as_bytes(), &mut out);
    out
}

fn visit(node: tree_sitter::Node, src: &[u8], out: &mut Vec<(String, usize)>) {
    let line = node.start_position().row + 1;

    match node.kind() {
        "decorator" => {
            let t = node.utf8_text(src).unwrap_or("");
            for d in ["login_required", "requires_auth", "auth_required",
                      "permission_required", "require_auth"] {
                if t.contains(d) {
                    out.push((format!("auth:{}", d), line));
                }
            }
        }
        "call" => {
            if let Some(func) = node.child_by_field_name("function") {
                let f = func.utf8_text(src).unwrap_or("");
                match f {
                    "subprocess.run" | "subprocess.call" | "subprocess.Popen" => {
                        if kwarg(node, "shell", src).as_deref() == Some("True") {
                            out.push(("call:subprocess:shell=True".into(), line));
                        }
                    }
                    "requests.get" | "requests.post" | "requests.put" | "requests.delete" => {
                        if kwarg(node, "verify", src).as_deref() == Some("False") {
                            out.push(("call:requests:verify=False".into(), line));
                        }
                    }
                    "eval" => out.push(("call:eval".into(), line)),
                    "exec" => out.push(("call:exec".into(), line)),
                    _ => {}
                }
            }
        }
        "assignment" => {
            if let (Some(l), Some(r)) = (
                node.child_by_field_name("left"),
                node.child_by_field_name("right"),
            ) {
                let lname = l.utf8_text(src).unwrap_or("").trim();
                let rtext = r.utf8_text(src).unwrap_or("").trim();
                if lname == "DEBUG" && rtext == "True" {
                    out.push(("config:DEBUG=True".into(), line));
                }
                if lname == "allow_origins"
                    && (rtext.contains("\"*\"") || rtext.contains("'*'"))
                {
                    out.push(("config:CORS=*".into(), line));
                }
            }
        }
        "import_statement" | "import_from_statement" => {
            let t = node.utf8_text(src).unwrap_or("");
            for d in ["pickle", "marshal"] {
                if t.contains(d) {
                    out.push((format!("import:{}", d), line));
                }
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit(child, src, out);
    }
}

fn kwarg(node: tree_sitter::Node, name: &str, src: &[u8]) -> Option<String> {
    let args = node.child_by_field_name("arguments")?;
    let mut cursor = args.walk();
    for child in args.children(&mut cursor) {
        if child.kind() == "keyword_argument" {
            if let (Some(k), Some(v)) = (
                child.child_by_field_name("name"),
                child.child_by_field_name("value"),
            ) {
                if k.utf8_text(src).ok()? == name {
                    return Some(v.utf8_text(src).ok()?.to_string());
                }
            }
        }
    }
    None
}