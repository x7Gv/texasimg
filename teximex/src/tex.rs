/// Represents an unvalidated character string that can be turned interpreted as a **TeX** token.
pub trait TexString {
    /// Apply implementation specific mutations and return a freshly allocated string.
    fn to_tex(&self) -> String;
}

impl TexString for &str {
    fn to_tex(&self) -> String {
        self.to_string()
    }
}

impl TexString for String {
    fn to_tex(&self) -> String {
        self.clone()
    }
}

/*
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Usepackage<T: TexString> {
    arbitrary: Vec<T>,
    main: T,
}
*/

/*
fn parse_usepackage(mut package: Usepackage<String>, string: String) -> (Usepackage<String>, String) {

    let split = string.splitn(2, "[").collect::<Vec<_>>();

    if split.len() == 1 {
        let ssplit = split[0].splitn(2, "{").collect::<Vec<_>>()[1].splitn(2, "}").collect::<Vec<_>>()[0];

        package.main = ssplit.to_string();

        return (package, "".to_string())
    } else {

    let spl = split[1].splitn(2, "]").collect::<Vec<_>>()[0];
    package.arbitrary.push(spl.to_string());

    return parse_usepackage(package, split[1].to_string())
    }
}

impl Usepackage<String> {

    pub fn from_str(string: &str) -> Self {

        let res = parse_usepackage(Usepackage::new("".to_string()), string.to_string());
        res.0
    }
}

*/

/// Represents a `\color` (La)TeX command.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Copy)]
pub enum Color {
    /// `\color{black}`
    Black,
    /// `color{white}`
    White,
}

impl TexString for Color {
    fn to_tex(&self) -> String {
        match self {
            Color::Black => r#"\color{black}"#.to_string(),
            Color::White => r#"\color{white}"#.to_string(),
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::Black
    }
}

/*
impl<T: TexString> Usepackage<T> {
    pub fn new(main: T) -> Self {
        Self {
            main,
            arbitrary: Vec::new(),
        }
    }

    pub fn new_with_arbitrary(main: T, arbitrary: Vec<T>) -> Self {
        Self { arbitrary, main }
    }
}

impl<T: TexString> TexString for Usepackage<T> {
    fn to_tex(&self) -> String {
        let mut out = String::from(r#"\usepackage"#);
        for tok in &self.arbitrary {
            out.push_str(&format!("[{}]", tok.to_tex()));
        }
        out.push_str(&format!("{{{}}}", self.main.to_tex()));
        out
    }
}

impl<T: TexString> TexString for Vec<Usepackage<T>> {
    fn to_tex(&self) -> String {
        let mut out = String::new();
        for tok in self {
            out.push_str(&format!("{}\n", tok.to_tex()));
        }
        out
    }
}
*/

/// Represents a (La)TeX MathMode token string.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MathMode<T: TexString> {
    /// Inline math mode i.e. `\( tok... \)`
    Inline(Vec<T>),
    /// Displayed math mode i.e. `\[ tok... \]`
    Displayed(Vec<T>),
}

impl<T: TexString> TexString for MathMode<T> {
    fn to_tex(&self) -> String {
        match self {
            MathMode::Inline(inline) => {
                let mut inner = String::new();

                for tok in inline.iter() {
                    inner.push_str(&tok.to_tex());
                }

                format!(r#"\( {} \)"#, inner)
            }
            MathMode::Displayed(displayed) => {
                let mut inner = String::new();

                for tok in displayed.iter() {
                    inner.push_str(&tok.to_tex());
                }

                format!(r#"\[ {} \]"#, inner)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    mod mathmode {
        use crate::tex::{MathMode, TexString};

        #[test]
        fn to_tex() {
            let mm1 = MathMode::Displayed(vec!["1+1"]);
            let mm2 = MathMode::Inline(vec![r#"\sqrt{2}"#, "i^2"]);

            assert_eq!(mm1.to_tex(), r#"\[ 1+1 \]"#);
            assert_eq!(mm2.to_tex(), r#"\( \sqrt{2}i^2 \)"#);
        }
    }
}
