use super::*;

    #[test]
    fn test_toast_data_builder() {
        let toast = ToastData::new("test_1", "Test Toast")
            .description("Description")
            .variant(ToastVariant::Success)
            .button(ToastButton::new("Action"));

        assert_eq!(toast.id, "test_1");
        assert_eq!(toast.title, "Test Toast");
        assert_eq!(toast.variant, ToastVariant::Success);
        assert_eq!(toast.buttons.len(), 1);
        assert_eq!(toast.count, 1);
    }
