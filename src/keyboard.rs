pub(crate) fn char_to_index(key: char) -> usize {
    match key {
        '0' => {
            return 0;
        }
        '1' => {
            return 1;
        }
        '2' => {
            return 2;
        }
        '3' => {
            return 3;
        }
        '4' => {
            return 4;
        }
        '5' => {
            return 5;
        }
        '6' => {
            return 6;
        }
        '7' => {
            return 7;
        }
        '8' => {
            return 8;
        }
        '9' => {
            return 9;
        }
        'A' => {
            return 10;
        }
        'B' => {
            return 11;
        }
        'C' => {
            return 12;
        }
        'D' => {
            return 13;
        }
        'E' => {
            return 14;
        }
        'F' => {
            return 15;
        }
        _ => {
            return 0;
        }
    }
}
