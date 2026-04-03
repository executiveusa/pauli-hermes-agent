from night_shift.policy import ActionClass, classify_action, requires_approval


def test_classify_delete_is_irreversible():
    assert classify_action("delete_branch") == ActionClass.IRREVERSIBLE_ACTION
    assert requires_approval("delete_branch") is True


def test_classify_write_safe_write():
    assert classify_action("write_file") == ActionClass.SAFE_WRITE
    assert requires_approval("write_file") is False


def test_classify_publish_sensitive():
    assert classify_action("publish_release") == ActionClass.SENSITIVE_WRITE
    assert requires_approval("publish_release") is True
