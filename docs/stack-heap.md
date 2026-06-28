NonNull::from and Node node; => stack allocation which means the memory is cleared after the function ends

NonNull::new_unchecked() and Node node = new Node() => heap allocation, the data persists even after function ends and persists untill delete is used