pub const fn wrap_decrement(index: usize, len: usize) -> usize {
    if len == 0 {
        return 0;
    }

    if index == 0 {
        len - 1
    } else {
        index - 1
    }
}

pub const fn wrap_increment(index: usize, len: usize) -> usize {
    if len == 0 {
        return 0;
    }

    (index + 1) % len
}
