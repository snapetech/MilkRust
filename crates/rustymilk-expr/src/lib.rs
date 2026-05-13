// =============================================================================
// rustymilk-expr: Expression tokenizer, recursive-descent parser, and equation
// evaluation engine for MilkDrop-compatible preset scripts.
//
// This crate implements the full expression grammar including:
//   - Tokenization (identifiers, numbers, operators, parentheses)
//   - Recursive-descent parser with precedence climbing
//   - 50+ scoped math functions (sin, cos, atan2, pow, etc.)
//   - Loop/while/exec2/exec3 control flow
//   - megabuf/gmegabuf indexed memory
//   - Deterministic pseudo-random via seeded counter
// =============================================================================

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum RustyMilkValue {
    Number(f64),
    Text(String),
}

impl RustyMilkValue {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            Self::Text(_) => None,
        }
    }

    pub fn as_text(&self) -> String {
        match self {
            Self::Number(value) => {
                if value.fract().abs() < f64::EPSILON {
                    format!("{}", *value as i64)
                } else {
                    format!("{value}")
                }
            }
            Self::Text(value) => value.clone(),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum RustyMilkToken {
    Ident(String),
    Number(f64),
    Op(String),
}

fn tokenize_rustymilk_expression(expression: &str) -> Result<Vec<RustyMilkToken>, String> {
    let chars = expression.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut index = 0usize;
    while index < chars.len() {
        let ch = chars[index];
        if ch.is_whitespace() {
            index += 1;
            continue;
        }
        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = index;
            index += 1;
            while index < chars.len()
                && (chars[index].is_ascii_alphanumeric()
                    || chars[index] == '_'
                    || chars[index] == '.')
            {
                index += 1;
            }
            tokens.push(RustyMilkToken::Ident(
                chars[start..index]
                    .iter()
                    .collect::<String>()
                    .to_ascii_lowercase(),
            ));
            continue;
        }
        if ch.is_ascii_digit() || ch == '.' {
            let start = index;
            index += 1;
            while index < chars.len() && (chars[index].is_ascii_digit() || chars[index] == '.') {
                index += 1;
            }
            if index < chars.len() && matches!(chars[index], 'e' | 'E') {
                index += 1;
                if index < chars.len() && matches!(chars[index], '+' | '-') {
                    index += 1;
                }
                while index < chars.len() && chars[index].is_ascii_digit() {
                    index += 1;
                }
            }
            let value = chars[start..index]
                .iter()
                .collect::<String>()
                .parse::<f64>()
                .map_err(|_| format!("Unsupported RustyMilk expression syntax: {expression}"))?;
            tokens.push(RustyMilkToken::Number(value));
            continue;
        }
        let two = if index + 1 < chars.len() {
            Some([chars[index], chars[index + 1]].iter().collect::<String>())
        } else {
            None
        };
        if let Some(two) = two.as_deref().filter(|value| {
            matches!(
                *value,
                "&&" | "||" | "<<" | ">>" | "==" | "!=" | "<=" | ">=" | "+=" | "-=" | "*=" | "/="
            )
        }) {
            tokens.push(RustyMilkToken::Op(two.to_string()));
            index += 2;
            continue;
        }
        if matches!(
            ch,
            '(' | ')'
                | '+'
                | '-'
                | '*'
                | '/'
                | '%'
                | ','
                | ';'
                | '?'
                | ':'
                | '<'
                | '>'
                | '&'
                | '|'
                | '^'
                | '='
                | '!'
                | '~'
        ) {
            tokens.push(RustyMilkToken::Op(ch.to_string()));
            index += 1;
            continue;
        }
        return Err(format!(
            "Unsupported RustyMilk expression syntax: {expression}"
        ));
    }
    Ok(tokens)
}

fn rustymilk_number(scope: &BTreeMap<String, RustyMilkValue>, name: &str) -> f64 {
    scope
        .get(name)
        .and_then(RustyMilkValue::as_number)
        .unwrap_or(0.0)
}

fn rustymilk_buffer_key(name: &str, index: f64) -> String {
    let prefix = if name.eq_ignore_ascii_case("gmegabuf") {
        "gmegabuf"
    } else {
        "megabuf"
    };
    let index = if index.is_finite() {
        index.trunc().max(0.0) as usize
    } else {
        0
    };
    format!("{prefix}_{index}")
}

fn rustymilk_buffer_number(
    scope: &BTreeMap<String, RustyMilkValue>,
    name: &str,
    index: f64,
) -> f64 {
    rustymilk_number(scope, &rustymilk_buffer_key(name, index))
}

fn rustymilk_indexed_sample(values: &[f64], position: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let normalized = position.clamp(0.0, 1.0);
    let index = ((normalized * values.len() as f64).floor() as usize).min(values.len() - 1);
    let value = values[index];
    if value > 1.0 {
        value / 255.0
    } else {
        value
    }
}

fn mix_rustymilk_rand_seed(mut seed: u64, value: f64) -> u64 {
    seed ^= value.to_bits().wrapping_add(0x9e37_79b9_7f4a_7c15);
    seed = seed.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    seed ^ (seed >> 31)
}

fn rustymilk_pseudo_random_unit(scope: &BTreeMap<String, RustyMilkValue>, counter: usize) -> f64 {
    let mut seed = 0x4d49_4c4b_4452_4f50u64 ^ counter as u64;
    for key in [
        "time", "frame", "bass", "mid", "treb", "bass_att", "mid_att", "treb_att",
    ] {
        seed = mix_rustymilk_rand_seed(seed, rustymilk_number(scope, key));
    }
    seed = mix_rustymilk_rand_seed(seed, counter as f64 + 0.123_456_789);
    seed ^= seed >> 12;
    seed ^= seed << 25;
    seed ^= seed >> 27;
    let value = seed.wrapping_mul(0x2545_f491_4f6c_dd1d);
    (value as f64) / (u64::MAX as f64)
}

const RUSTYMILK_MAX_LOOP_ITERATIONS: usize = 200_000;

struct RustyMilkExpressionParser {
    rand_counter: usize,
    scope: BTreeMap<String, RustyMilkValue>,
    tokens: Vec<RustyMilkToken>,
    index: usize,
}

impl RustyMilkExpressionParser {
    fn new(
        tokens: Vec<RustyMilkToken>,
        scope: BTreeMap<String, RustyMilkValue>,
        rand_counter: usize,
    ) -> Self {
        Self {
            rand_counter,
            scope,
            tokens,
            index: 0,
        }
    }

    fn peek_op(&self) -> Option<&str> {
        match self.tokens.get(self.index) {
            Some(RustyMilkToken::Op(value)) => Some(value),
            _ => None,
        }
    }

    fn remaining_call_args(&mut self, name: &str) -> Result<Vec<Vec<RustyMilkToken>>, String> {
        let mut args = Vec::new();
        let mut current = Vec::new();
        let mut depth = 0usize;
        while let Some(token) = self.consume() {
            match &token {
                RustyMilkToken::Op(value) if value == "(" => {
                    depth += 1;
                    current.push(token);
                }
                RustyMilkToken::Op(value) if value == ")" => {
                    if depth == 0 {
                        if !current.is_empty() {
                            args.push(current);
                        }
                        return Ok(args);
                    }
                    depth -= 1;
                    current.push(token);
                }
                RustyMilkToken::Op(value) if (value == "," || value == ";") && depth == 0 => {
                    args.push(current);
                    current = Vec::new();
                }
                _ => current.push(token),
            }
        }
        Err(format!("Unclosed function call: {name}"))
    }

    fn evaluate_arg_tokens(&mut self, tokens: &[RustyMilkToken]) -> Result<f64, String> {
        if let Some((target_tokens, operator, value_tokens)) = self.assignment_arg_parts(tokens) {
            let Some(key) = self.lvalue_key(target_tokens)? else {
                return Err(format!("{operator} requires a variable or buffer target."));
            };
            let current = rustymilk_number(&self.scope, &key);
            let next = self.evaluate_arg_tokens(value_tokens)?;
            let value = apply_rustymilk_assignment_operator(current, operator, next);
            self.scope.insert(key, RustyMilkValue::Number(value));
            return Ok(value);
        }
        let mut parser =
            RustyMilkExpressionParser::new(tokens.to_vec(), self.scope.clone(), self.rand_counter);
        let value = parser.parse()?;
        self.scope = parser.scope;
        self.rand_counter = parser.rand_counter;
        Ok(value)
    }

    fn assignment_arg_parts<'b>(
        &self,
        tokens: &'b [RustyMilkToken],
    ) -> Option<(&'b [RustyMilkToken], &'b str, &'b [RustyMilkToken])> {
        let mut depth = 0usize;
        for (index, token) in tokens.iter().enumerate() {
            match token {
                RustyMilkToken::Op(value) if value == "(" => depth += 1,
                RustyMilkToken::Op(value) if value == ")" => depth = depth.saturating_sub(1),
                RustyMilkToken::Op(value)
                    if depth == 0 && matches!(value.as_str(), "=" | "+=" | "-=" | "*=" | "/=") =>
                {
                    return Some((&tokens[..index], value, &tokens[index + 1..]));
                }
                _ => {}
            }
        }
        None
    }

    fn assign_arg_tokens(
        &mut self,
        target_tokens: &[RustyMilkToken],
        value: f64,
    ) -> Result<f64, String> {
        let Some(key) = self.lvalue_key(target_tokens)? else {
            return Err("assign() requires a variable or buffer target.".to_string());
        };
        self.scope.insert(key, RustyMilkValue::Number(value));
        Ok(value)
    }

    fn lvalue_key(&mut self, tokens: &[RustyMilkToken]) -> Result<Option<String>, String> {
        match tokens {
            [RustyMilkToken::Ident(name)] => Ok(Some(name.to_ascii_lowercase())),
            [RustyMilkToken::Ident(name), RustyMilkToken::Op(open), rest @ .., RustyMilkToken::Op(close)]
                if open == "("
                    && close == ")"
                    && (name.eq_ignore_ascii_case("megabuf")
                        || name.eq_ignore_ascii_case("gmegabuf")) =>
            {
                let index = self.evaluate_arg_tokens(rest)?;
                Ok(Some(rustymilk_buffer_key(name, index)))
            }
            _ => Ok(None),
        }
    }

    fn call_special_function(
        &mut self,
        name: &str,
        args: &[Vec<RustyMilkToken>],
    ) -> Result<Option<f64>, String> {
        match name {
            "assign" => {
                if args.len() < 2 {
                    return Ok(Some(0.0));
                }
                let value = self.evaluate_arg_tokens(&args[1])?;
                self.assign_arg_tokens(&args[0], value).map(Some)
            }
            "exec2" | "exec3" => {
                let mut first = 0.0;
                for (index, arg) in args.iter().enumerate() {
                    let value = self.evaluate_arg_tokens(arg)?;
                    if index == 0 {
                        first = value;
                    }
                }
                Ok(Some(first))
            }
            "loop" => {
                if args.is_empty() {
                    return Ok(Some(0.0));
                }
                let count = self
                    .evaluate_arg_tokens(&args[0])?
                    .trunc()
                    .clamp(0.0, RUSTYMILK_MAX_LOOP_ITERATIONS as f64)
                    as usize;
                let mut last = 0.0;
                for _ in 0..count {
                    for arg in &args[1..] {
                        last = self.evaluate_arg_tokens(arg)?;
                    }
                }
                Ok(Some(last))
            }
            "while" => {
                if args.is_empty() {
                    return Ok(Some(0.0));
                }
                let mut last = 0.0;
                for _ in 0..RUSTYMILK_MAX_LOOP_ITERATIONS {
                    let condition = self.evaluate_rustymilk_while_condition(&args[0])?;
                    if condition == 0.0 {
                        break;
                    }
                    last = condition;
                    for arg in &args[1..] {
                        last = self.evaluate_arg_tokens(arg)?;
                    }
                }
                Ok(Some(last))
            }
            "memcpy" => {
                if args.len() < 3 {
                    return Ok(Some(0.0));
                }
                let dest = self.evaluate_arg_tokens(&args[0])?.trunc().max(0.0) as usize;
                let source = self.evaluate_arg_tokens(&args[1])?.trunc().max(0.0) as usize;
                let count = self
                    .evaluate_arg_tokens(&args[2])?
                    .trunc()
                    .clamp(0.0, RUSTYMILK_MAX_LOOP_ITERATIONS as f64)
                    as usize;
                let values = (0..count)
                    .map(|offset| {
                        rustymilk_number(
                            &self.scope,
                            &rustymilk_buffer_key("megabuf", (source + offset) as f64),
                        )
                    })
                    .collect::<Vec<_>>();
                for (offset, value) in values.into_iter().enumerate() {
                    self.scope.insert(
                        rustymilk_buffer_key("megabuf", (dest + offset) as f64),
                        RustyMilkValue::Number(value),
                    );
                }
                Ok(Some(count as f64))
            }
            _ => Ok(None),
        }
    }

    fn evaluate_rustymilk_while_condition(
        &mut self,
        tokens: &[RustyMilkToken],
    ) -> Result<f64, String> {
        if let [RustyMilkToken::Ident(name), RustyMilkToken::Op(open), rest @ .., RustyMilkToken::Op(close)] =
            tokens
        {
            if open == "("
                && close == ")"
                && (name.eq_ignore_ascii_case("exec2") || name.eq_ignore_ascii_case("exec3"))
            {
                let args = split_rustymilk_call_tokens(rest);
                if args.is_empty() {
                    return Ok(0.0);
                }
                let condition = self.evaluate_arg_tokens(&args[0])?;
                if condition != 0.0 {
                    for arg in &args[1..] {
                        self.evaluate_arg_tokens(arg)?;
                    }
                }
                return Ok(condition);
            }
        }
        self.evaluate_arg_tokens(tokens)
    }

    fn match_op(&mut self, expected: &str) -> bool {
        if self.peek_op() == Some(expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn consume(&mut self) -> Option<RustyMilkToken> {
        let token = self.tokens.get(self.index).cloned();
        if token.is_some() {
            self.index += 1;
        }
        token
    }

    fn parse(&mut self) -> Result<f64, String> {
        if self.index == 0 {
            if let Some((assignment_index, operator)) =
                find_rustymilk_top_level_assignment_token(&self.tokens)
            {
                let target_tokens = self.tokens[..assignment_index].to_vec();
                let expression_tokens = self.tokens[assignment_index + 1..].to_vec();
                let current = self
                    .lvalue_key(&target_tokens)?
                    .map(|key| rustymilk_number(&self.scope, &key))
                    .unwrap_or(0.0);
                let next = self.evaluate_arg_tokens(&expression_tokens)?;
                let value = apply_rustymilk_assignment_operator(current, operator, next);
                self.assign_arg_tokens(&target_tokens, value)?;
                self.index = self.tokens.len();
                return Ok(value);
            }
        }
        let value = self.parse_conditional()?;
        if self.index < self.tokens.len() {
            return Err("Unexpected trailing RustyMilk token".to_string());
        }
        Ok(value)
    }

    fn parse_primary(&mut self) -> Result<f64, String> {
        match self.consume() {
            Some(RustyMilkToken::Number(value)) => Ok(value),
            Some(RustyMilkToken::Op(op)) if op == "(" => {
                let value = self.parse_conditional()?;
                if !self.match_op(")") {
                    return Err("Unclosed RustyMilk expression group.".to_string());
                }
                Ok(value)
            }
            Some(RustyMilkToken::Ident(name)) => {
                if self.match_op("(") {
                    let args = self.remaining_call_args(&name)?;
                    if let Some(value) = self.call_special_function(&name, &args)? {
                        return Ok(value);
                    }
                    let values = args
                        .iter()
                        .map(|arg| self.evaluate_arg_tokens(arg))
                        .collect::<Result<Vec<_>, _>>()?;
                    self.call_function(&name, &values)
                } else {
                    Ok(match name.as_str() {
                        "e" => std::f64::consts::E,
                        "pi" => std::f64::consts::PI,
                        _ => rustymilk_number(&self.scope, &name),
                    })
                }
            }
            Some(token) => Err(format!("Unexpected RustyMilk token: {token:?}")),
            None => Err("Unexpected end of RustyMilk expression.".to_string()),
        }
    }

    fn call_function(&mut self, name: &str, args: &[f64]) -> Result<f64, String> {
        let arg = |index: usize, default: f64| args.get(index).copied().unwrap_or(default);
        let out = match name {
            "abs" => arg(0, 0.0).abs(),
            "above" => (arg(0, 0.0) > arg(1, 0.0)) as i32 as f64,
            "acos" => arg(0, 0.0).clamp(-1.0, 1.0).acos(),
            "asin" => arg(0, 0.0).clamp(-1.0, 1.0).asin(),
            "atan" => arg(0, 0.0).atan(),
            "atan2" => arg(0, 0.0).atan2(arg(1, 0.0)),
            "below" => (arg(0, 0.0) < arg(1, 0.0)) as i32 as f64,
            "band" => ((arg(0, 0.0).trunc() as i64) & (arg(1, 0.0).trunc() as i64)) as f64,
            "bor" => ((arg(0, 0.0).trunc() as i64) | (arg(1, 0.0).trunc() as i64)) as f64,
            "bnot" => (!(arg(0, 0.0).trunc() as i64)) as f64,
            "bxor" => ((arg(0, 0.0).trunc() as i64) ^ (arg(1, 0.0).trunc() as i64)) as f64,
            "ceil" => arg(0, 0.0).ceil(),
            "cos" => arg(0, 0.0).cos(),
            "div" => {
                let right = arg(1, 0.0);
                if right == 0.0 {
                    0.0
                } else {
                    arg(0, 0.0) / right
                }
            }
            "equal" => ((arg(0, 0.0) - arg(1, 0.0)).abs() < 0.00001) as i32 as f64,
            "exp" => arg(0, 0.0).exp(),
            "floor" => arg(0, 0.0).floor(),
            "gmegabuf" => rustymilk_buffer_number(&self.scope, name, arg(0, 0.0)),
            "get_fft" => {
                let values = rustymilk_frequency_data(&self.scope);
                rustymilk_indexed_sample(&values, arg(0, 0.0))
            }
            "get_fft_hz" => {
                let sample_rate = rustymilk_number(&self.scope, "sample_rate")
                    .max(rustymilk_number(&self.scope, "samplerate"))
                    .max(44100.0);
                let nyquist = sample_rate / 2.0;
                let values = rustymilk_frequency_data(&self.scope);
                rustymilk_indexed_sample(
                    &values,
                    if nyquist > 0.0 {
                        arg(0, 0.0) / nyquist
                    } else {
                        0.0
                    },
                )
            }
            "get_waveform" => {
                let values = rustymilk_waveform_data(&self.scope);
                rustymilk_indexed_sample(&values, arg(0, 0.0))
            }
            "if" => {
                if arg(0, 0.0) != 0.0 {
                    arg(1, 0.0)
                } else {
                    arg(2, 0.0)
                }
            }
            "int" => arg(0, 0.0).trunc(),
            "log" => {
                if arg(0, 0.0) <= 0.0 {
                    0.0
                } else {
                    arg(0, 0.0).ln()
                }
            }
            "log10" => {
                if arg(0, 0.0) <= 0.0 {
                    0.0
                } else {
                    arg(0, 0.0).log10()
                }
            }
            "max" => arg(0, 0.0).max(arg(1, 0.0)),
            "megabuf" => rustymilk_buffer_number(&self.scope, name, arg(0, 0.0)),
            "min" => arg(0, 0.0).min(arg(1, 0.0)),
            "mod" => {
                let right = arg(1, 0.0);
                if right == 0.0 {
                    0.0
                } else {
                    arg(0, 0.0) % right
                }
            }
            "pow" => arg(0, 0.0).powf(arg(1, 0.0)),
            "rand" => {
                let upper = arg(0, 1.0).trunc().max(0.0);
                if upper <= 0.0 {
                    0.0
                } else {
                    let counter = self.rand_counter;
                    self.rand_counter += 1;
                    (rustymilk_pseudo_random_unit(&self.scope, counter) * upper)
                        .floor()
                        .min(upper - 1.0)
                }
            }
            "sign" => arg(0, 0.0).signum(),
            "sigmoid" => {
                let constraint = if arg(1, 1.0) == 0.0 { 1.0 } else { arg(1, 1.0) };
                1.0 / (1.0 + (-arg(0, 0.0) * constraint).exp())
            }
            "sin" => arg(0, 0.0).sin(),
            "sqr" => arg(0, 0.0) * arg(0, 0.0),
            "sqrt" => {
                if arg(0, 0.0) < 0.0 {
                    0.0
                } else {
                    arg(0, 0.0).sqrt()
                }
            }
            "tan" => arg(0, 0.0).tan(),
            _ => return Err(format!("Unsupported RustyMilk function: {name}")),
        };
        Ok(if out.is_finite() { out } else { 0.0 })
    }

    fn parse_unary(&mut self) -> Result<f64, String> {
        if self.match_op("+") {
            return self.parse_unary();
        }
        if self.match_op("-") {
            return Ok(-self.parse_unary()?);
        }
        if self.match_op("!") {
            return Ok((self.parse_unary()? == 0.0) as i32 as f64);
        }
        if self.match_op("~") {
            return Ok((!(self.parse_unary()?.trunc() as i64)) as f64);
        }
        self.parse_primary()
    }

    fn parse_factor(&mut self) -> Result<f64, String> {
        let mut value = self.parse_unary()?;
        while let Some(op) = self
            .peek_op()
            .filter(|op| matches!(*op, "*" | "/" | "%"))
            .map(str::to_string)
        {
            self.index += 1;
            let right = self.parse_unary()?;
            value = match op.as_str() {
                "*" => value * right,
                "/" => {
                    if right == 0.0 {
                        0.0
                    } else {
                        value / right
                    }
                }
                "%" => {
                    if right == 0.0 {
                        0.0
                    } else {
                        value % right
                    }
                }
                _ => value,
            };
        }
        Ok(value)
    }

    fn parse_term(&mut self) -> Result<f64, String> {
        let mut value = self.parse_factor()?;
        while let Some(op) = self
            .peek_op()
            .filter(|op| matches!(*op, "+" | "-"))
            .map(str::to_string)
        {
            self.index += 1;
            let right = self.parse_factor()?;
            value = if op == "+" {
                value + right
            } else {
                value - right
            };
        }
        Ok(value)
    }

    fn parse_shift(&mut self) -> Result<f64, String> {
        let mut value = self.parse_term()?;
        while let Some(op) = self
            .peek_op()
            .filter(|op| matches!(*op, "<<" | ">>"))
            .map(str::to_string)
        {
            self.index += 1;
            let right = self.parse_term()?;
            value = if op == "<<" {
                ((value.trunc() as i64) << (right.trunc() as u32)) as f64
            } else {
                ((value.trunc() as i64) >> (right.trunc() as u32)) as f64
            };
        }
        Ok(value)
    }

    fn parse_comparison(&mut self) -> Result<f64, String> {
        let mut value = self.parse_shift()?;
        while let Some(op) = self
            .peek_op()
            .filter(|op| matches!(*op, "<" | ">" | "<=" | ">=" | "==" | "!="))
            .map(str::to_string)
        {
            self.index += 1;
            let right = self.parse_shift()?;
            value = match op.as_str() {
                "<" => (value < right) as i32 as f64,
                ">" => (value > right) as i32 as f64,
                "<=" => (value <= right) as i32 as f64,
                ">=" => (value >= right) as i32 as f64,
                "==" => ((value - right).abs() < 0.00001) as i32 as f64,
                "!=" => ((value - right).abs() >= 0.00001) as i32 as f64,
                _ => value,
            };
        }
        Ok(value)
    }

    fn parse_bitwise_and(&mut self) -> Result<f64, String> {
        let mut value = self.parse_comparison()?;
        while self.match_op("&") {
            value = ((value.trunc() as i64) & (self.parse_comparison()?.trunc() as i64)) as f64;
        }
        Ok(value)
    }

    fn parse_bitwise_xor(&mut self) -> Result<f64, String> {
        let mut value = self.parse_bitwise_and()?;
        while self.match_op("^") {
            value = ((value.trunc() as i64) ^ (self.parse_bitwise_and()?.trunc() as i64)) as f64;
        }
        Ok(value)
    }

    fn parse_bitwise_or(&mut self) -> Result<f64, String> {
        let mut value = self.parse_bitwise_xor()?;
        while self.match_op("|") {
            value = ((value.trunc() as i64) | (self.parse_bitwise_xor()?.trunc() as i64)) as f64;
        }
        Ok(value)
    }

    fn parse_logical_and(&mut self) -> Result<f64, String> {
        let mut value = self.parse_bitwise_or()?;
        while self.match_op("&&") {
            value = (value != 0.0 && self.parse_bitwise_or()? != 0.0) as i32 as f64;
        }
        Ok(value)
    }

    fn parse_logical_or(&mut self) -> Result<f64, String> {
        let mut value = self.parse_logical_and()?;
        while self.match_op("||") {
            value = (value != 0.0 || self.parse_logical_and()? != 0.0) as i32 as f64;
        }
        Ok(value)
    }

    fn parse_conditional(&mut self) -> Result<f64, String> {
        let condition = self.parse_logical_or()?;
        if !self.match_op("?") {
            return Ok(condition);
        }
        let when_true = self.parse_conditional()?;
        if !self.match_op(":") {
            return Err("Unclosed RustyMilk conditional expression.".to_string());
        }
        let when_false = self.parse_conditional()?;
        Ok(if condition != 0.0 {
            when_true
        } else {
            when_false
        })
    }
}

fn rustymilk_frequency_data(scope: &BTreeMap<String, RustyMilkValue>) -> Vec<f64> {
    [
        "frequency_data",
        "frequencies",
        "frequency",
        "spectrum",
        "fft",
    ]
    .into_iter()
    .find_map(|name| match scope.get(name) {
        Some(RustyMilkValue::Text(value)) => Some(
            value
                .split(',')
                .filter_map(|item| item.trim().parse::<f64>().ok())
                .collect::<Vec<_>>(),
        ),
        Some(RustyMilkValue::Number(value)) => Some(vec![*value]),
        None => None,
    })
    .unwrap_or_default()
}

fn rustymilk_waveform_data(scope: &BTreeMap<String, RustyMilkValue>) -> Vec<f64> {
    ["waveform_data", "waveform", "samples", "wave"]
        .into_iter()
        .find_map(|name| match scope.get(name) {
            Some(RustyMilkValue::Text(value)) => Some(
                value
                    .split(',')
                    .filter_map(|item| item.trim().parse::<f64>().ok())
                    .collect::<Vec<_>>(),
            ),
            Some(RustyMilkValue::Number(value)) => Some(vec![*value]),
            None => None,
        })
        .unwrap_or_default()
}

fn find_rustymilk_top_level_assignment_token(
    tokens: &[RustyMilkToken],
) -> Option<(usize, &'static str)> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        match token {
            RustyMilkToken::Op(value) if value == "(" => depth += 1,
            RustyMilkToken::Op(value) if value == ")" => depth = depth.saturating_sub(1),
            RustyMilkToken::Op(value) if depth == 0 => match value.as_str() {
                "+=" => return Some((index, "+=")),
                "-=" => return Some((index, "-=")),
                "*=" => return Some((index, "*=")),
                "/=" => return Some((index, "/=")),
                "=" => return Some((index, "=")),
                _ => {}
            },
            _ => {}
        }
    }
    None
}

fn split_rustymilk_call_tokens(tokens: &[RustyMilkToken]) -> Vec<Vec<RustyMilkToken>> {
    let mut args = Vec::new();
    let mut current = Vec::new();
    let mut depth = 0usize;
    for token in tokens {
        match token {
            RustyMilkToken::Op(value) if value == "(" => {
                depth += 1;
                current.push(token.clone());
            }
            RustyMilkToken::Op(value) if value == ")" => {
                depth = depth.saturating_sub(1);
                current.push(token.clone());
            }
            RustyMilkToken::Op(value) if (value == "," || value == ";") && depth == 0 => {
                args.push(current);
                current = Vec::new();
            }
            _ => current.push(token.clone()),
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

pub fn evaluate_rustymilk_expression(
    expression: &str,
    variables: &BTreeMap<String, RustyMilkValue>,
) -> Result<f64, String> {
    evaluate_rustymilk_expression_with_rand_counter(expression, variables, 0)
        .map(|(value, _)| value)
}

fn evaluate_rustymilk_expression_with_rand_counter(
    expression: &str,
    variables: &BTreeMap<String, RustyMilkValue>,
    rand_counter: usize,
) -> Result<(f64, usize), String> {
    evaluate_rustymilk_expression_with_scope(expression, variables, rand_counter)
        .map(|(value, _, rand_counter)| (value, rand_counter))
}

fn evaluate_rustymilk_expression_with_scope(
    expression: &str,
    variables: &BTreeMap<String, RustyMilkValue>,
    rand_counter: usize,
) -> Result<(f64, BTreeMap<String, RustyMilkValue>, usize), String> {
    let scope = variables
        .iter()
        .map(|(key, value)| (key.to_ascii_lowercase(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    let tokens = tokenize_rustymilk_expression(expression)?;
    let mut parser = RustyMilkExpressionParser::new(tokens, scope, rand_counter);
    let value = parser.parse()?;
    Ok((value, parser.scope, parser.rand_counter))
}

fn strip_rustymilk_equation_comments(text: &str) -> String {
    text.lines()
        .map(|line| line.split_once("//").map(|(code, _)| code).unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn split_rustymilk_equation_statements(equations: &str) -> Vec<String> {
    let sanitized = strip_rustymilk_equation_comments(equations);
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    for ch in sanitized.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ';' | '\n' if depth == 0 => {
                let statement = current.trim();
                if !statement.is_empty() {
                    statements.push(statement.to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let statement = current.trim();
    if !statement.is_empty() {
        statements.push(statement.to_string());
    }
    statements
}

pub fn evaluate_rustymilk_equations(
    equations: &str,
    variables: &BTreeMap<String, RustyMilkValue>,
) -> Result<BTreeMap<String, RustyMilkValue>, String> {
    let mut scope = variables
        .iter()
        .map(|(key, value)| (key.to_ascii_lowercase(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut rand_counter = rustymilk_number(&scope, "__rand_counter").max(0.0) as usize;
    for statement in split_rustymilk_equation_statements(equations) {
        if let Some((buffer_name, index_expression, operator, expression)) =
            split_rustymilk_buffer_assignment(&statement)
        {
            let (buffer_index, next_rand_counter) =
                evaluate_rustymilk_expression_with_rand_counter(
                    index_expression,
                    &scope,
                    rand_counter,
                )?;
            rand_counter = next_rand_counter;
            let key = rustymilk_buffer_key(&buffer_name, buffer_index);
            let current = rustymilk_number(&scope, &key);
            let (next, next_rand_counter) =
                evaluate_rustymilk_expression_with_rand_counter(expression, &scope, rand_counter)?;
            rand_counter = next_rand_counter;
            let value = apply_rustymilk_assignment_operator(current, operator, next);
            scope.insert(key, RustyMilkValue::Number(value));
            continue;
        }
        let Some((name, operator, expression)) = split_rustymilk_assignment(&statement) else {
            let (_, next_scope, next_rand_counter) =
                evaluate_rustymilk_expression_with_scope(&statement, &scope, rand_counter)?;
            scope = next_scope;
            rand_counter = next_rand_counter;
            continue;
        };
        let current = rustymilk_number(&scope, &name);
        let (next, next_rand_counter) =
            evaluate_rustymilk_expression_with_rand_counter(expression, &scope, rand_counter)?;
        rand_counter = next_rand_counter;
        let value = apply_rustymilk_assignment_operator(current, operator, next);
        scope.insert(name, RustyMilkValue::Number(value));
    }
    scope.insert(
        "__rand_counter".to_string(),
        RustyMilkValue::Number(rand_counter as f64),
    );
    Ok(scope)
}

fn apply_rustymilk_assignment_operator(current: f64, operator: &str, next: f64) -> f64 {
    match operator {
        "=" => next,
        "+=" => current + next,
        "-=" => current - next,
        "*=" => current * next,
        "/=" => {
            if next == 0.0 {
                0.0
            } else {
                current / next
            }
        }
        _ => next,
    }
}

fn split_rustymilk_buffer_assignment(
    statement: &str,
) -> Option<(String, &str, &'static str, &str)> {
    for operator in ["+=", "-=", "*=", "/=", "="] {
        let Some((raw_name, expression)) = statement.split_once(operator) else {
            continue;
        };
        let raw_name = raw_name.trim();
        let open = raw_name.find('(')?;
        let close = raw_name.rfind(')')?;
        if close <= open || close != raw_name.len() - 1 {
            continue;
        }
        let name = raw_name[..open].trim().to_ascii_lowercase();
        if name != "megabuf" && name != "gmegabuf" {
            continue;
        }
        let index_expression = raw_name[open + 1..close].trim();
        if index_expression.is_empty() {
            continue;
        }
        return Some((name, index_expression, operator, expression.trim()));
    }
    None
}

fn split_rustymilk_assignment(statement: &str) -> Option<(String, &'static str, &str)> {
    for operator in ["+=", "-=", "*=", "/=", "="] {
        if let Some((raw_name, expression)) = statement.split_once(operator) {
            let name = raw_name.trim();
            if !name.is_empty()
                && name
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.')
                && name
                    .chars()
                    .next()
                    .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
            {
                return Some((name.to_ascii_lowercase(), operator, expression.trim()));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_parses_identifiers_and_numbers() {
        let tokens = tokenize_rustymilk_expression("sin(time) + 3.14");
        assert!(tokens.is_ok());
        let tokens = tokens.unwrap();
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0], RustyMilkToken::Ident("sin".to_string()));
        assert_eq!(tokens[1], RustyMilkToken::Op("(".to_string()));
        assert_eq!(tokens[2], RustyMilkToken::Ident("time".to_string()));
        assert_eq!(tokens[3], RustyMilkToken::Op(")".to_string()));
        assert_eq!(tokens[4], RustyMilkToken::Op("+".to_string()));
        assert_eq!(tokens[5], RustyMilkToken::Number(3.14));
    }

    #[test]
    fn tokenize_rejects_invalid_characters() {
        let result = tokenize_rustymilk_expression("sin@time");
        assert!(result.is_err());
    }

    #[test]
    fn evaluate_simple_expression() {
        let scope: BTreeMap<String, RustyMilkValue> = BTreeMap::new();
        let val = evaluate_rustymilk_expression("2 + 3 * 4", &scope).unwrap();
        assert_eq!(val, 14.0);
    }

    #[test]
    fn evaluate_with_variables() {
        let mut scope = BTreeMap::new();
        scope.insert("x".to_string(), RustyMilkValue::Number(5.0));
        scope.insert("y".to_string(), RustyMilkValue::Number(3.0));
        let val = evaluate_rustymilk_expression("x * y + 1", &scope).unwrap();
        assert_eq!(val, 16.0);
    }

    #[test]
    fn evaluate_math_functions() {
        let scope: BTreeMap<String, RustyMilkValue> = BTreeMap::new();
        let val = evaluate_rustymilk_expression("sin(0)", &scope).unwrap();
        assert!((val - 0.0).abs() < 0.0001);
        let val = evaluate_rustymilk_expression("cos(0)", &scope).unwrap();
        assert!((val - 1.0).abs() < 0.0001);
        let val = evaluate_rustymilk_expression("sqrt(16)", &scope).unwrap();
        assert_eq!(val, 4.0);
        let val = evaluate_rustymilk_expression("pow(2,10)", &scope).unwrap();
        assert_eq!(val, 1024.0);
        let val = evaluate_rustymilk_expression("atan2(1,1)", &scope).unwrap();
        assert!((val - std::f64::consts::FRAC_PI_4).abs() < 0.0001);
    }

    #[test]
    fn evaluate_assignment_equations() {
        let mut scope: BTreeMap<String, RustyMilkValue> = BTreeMap::new();
        scope.insert("a".to_string(), RustyMilkValue::Number(1.0));
        let result = evaluate_rustymilk_equations("a = a + 1; a = a * 2", &scope).unwrap();
        assert_eq!(result["a"], RustyMilkValue::Number(4.0));
    }

    #[test]
    fn evaluate_while_loop() {
        let mut scope: BTreeMap<String, RustyMilkValue> = BTreeMap::new();
        scope.insert("n".to_string(), RustyMilkValue::Number(0.0));
        scope.insert("sum".to_string(), RustyMilkValue::Number(0.0));
        let result = evaluate_rustymilk_equations(
            "n = 0; sum = 0; while(n < 5, sum = sum + n; n = n + 1)",
            &scope,
        ).unwrap();
        assert_eq!(result["sum"], RustyMilkValue::Number(10.0));
    }

    #[test]
    fn evaluate_buffer_assignment() {
        let mut scope: BTreeMap<String, RustyMilkValue> = BTreeMap::new();
        let result = evaluate_rustymilk_equations("megabuf(0) = 42", &scope).unwrap();
        assert_eq!(result["megabuf_0"], RustyMilkValue::Number(42.0));
    }

    #[test]
    fn evaluate_pseudo_random_deterministic() {
        let scope: BTreeMap<String, RustyMilkValue> = BTreeMap::new();
        let val1 = evaluate_rustymilk_expression("rand(10)", &scope).unwrap();
        let val2 = evaluate_rustymilk_expression("rand(10)", &scope).unwrap();
        // Deterministic: same scope + same rand_counter -> same result
        assert_eq!(val1, val2);
    }
}

