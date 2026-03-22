const HIDDEN_VIRTUAL_ADAPTER_TOKENS: [&str; 3] = ["wsl", "hyper-v", "vethernet"];

fn contains_hidden_virtual_adapter_token(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    HIDDEN_VIRTUAL_ADAPTER_TOKENS
        .iter()
        .any(|token| lowered.contains(token))
}

pub fn is_hidden_virtual_adapter_name(name: &str, adapter_description: &str) -> bool {
    contains_hidden_virtual_adapter_token(name)
        || contains_hidden_virtual_adapter_token(adapter_description)
}

pub fn is_hidden_virtual_adapter_label(name: &str) -> bool {
    contains_hidden_virtual_adapter_token(name)
}
