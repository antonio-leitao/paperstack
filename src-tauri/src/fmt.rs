pub trait Clean {
    fn clean(&self) -> Self;
}

impl Clean for String {
    fn clean(&self) -> Self {
        let cleaned_str = self
            .chars() // Get iterator over characters
            .filter(|&c| !c.is_control()) // Filter out control characters (e.g., newline, tab)
            .collect(); // Collect the characters back into a String
        cleaned_str
    }
}
