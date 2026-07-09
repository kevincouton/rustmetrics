pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn unused_helper() -> i32 {
    42
}

pub fn large_function() -> i32 {
    let mut sum = 0;
    sum += 1;
    sum += 2;
    sum += 3;
    sum += 4;
    sum += 5;
    sum += 6;
    sum += 7;
    sum += 8;
    sum += 9;
    sum += 10;
    sum
}
