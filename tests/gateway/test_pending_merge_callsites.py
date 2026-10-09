"""Regression tests for gateway.run's use of merge_pending_message_event.

gateway/run.py used to define its own ``merge_pending_message_event(adapter,
...)`` that shadowed the platforms.base helper. Every caller passes the
adapter's pending dict, so the shadow silently queued nothing and the
``merge_text=`` callers raised TypeError. These tests pin the semantics of the
base helper at each call shape used in run.py. Stubbed only, no network.
"""
from unittest.mock import MagicMock

import gateway.run as run_mod
from gateway.platforms import base as base_mod
from gateway.platforms.base import MessageEvent, MessageType, SessionSource


_PLATFORM = MagicMock(value="telegram")


def _event(text="", message_type=MessageType.TEXT, media=None):
    source = SessionSource(
        platform=_PLATFORM,
        chat_id="123",
        chat_type="private",
        user_id="user1",
    )
    return MessageEvent(
        text=text,
        message_type=message_type,
        source=source,
        message_id="m",
        media_urls=list(media or []),
        media_types=["image/jpeg" for _ in (media or [])],
    )


def test_run_uses_the_base_helper_not_a_local_shadow():
    assert run_mod.merge_pending_message_event is base_mod.merge_pending_message_event


def test_generic_queue_or_replace_default_replaces_text():
    """run.py:_queue_or_replace_pending_event (no merge_text): a second text
    replaces the first queued text. Pins the current default semantics."""
    runner = object.__new__(run_mod.GatewayRunner)
    adapter = MagicMock()
    adapter._pending_messages = {}
    first, second = _event("first"), _event("second")
    runner.adapters = {first.source.platform: adapter}

    runner._queue_or_replace_pending_event("sk", first)
    assert adapter._pending_messages["sk"] is first  # actually queued (shadow dropped it)
    runner._queue_or_replace_pending_event("sk", second)
    assert adapter._pending_messages["sk"] is second
    assert adapter._pending_messages["sk"].text == "second"


def test_priority_photo_followup_media_always_merges():
    """run.py PHOTO priority follow-up (no merge_text): media merges even
    though text-only replace is the default."""
    pending = {}
    first = _event("caption one", MessageType.PHOTO, media=["a.jpg"])
    second = _event("caption two", MessageType.PHOTO, media=["b.jpg"])
    base_mod.merge_pending_message_event(pending, "sk", first)
    base_mod.merge_pending_message_event(pending, "sk", second)
    merged = pending["sk"]
    assert merged.media_urls == ["a.jpg", "b.jpg"]
    assert "caption one" in merged.text and "caption two" in merged.text


def test_text_then_photo_merges_media_into_pending_text():
    pending = {}
    base_mod.merge_pending_message_event(pending, "sk", _event("look at this"))
    base_mod.merge_pending_message_event(
        pending, "sk", _event("", MessageType.PHOTO, media=["a.jpg"])
    )
    merged = pending["sk"]
    assert merged.text == "look at this"
    assert merged.media_urls == ["a.jpg"]
    assert merged.message_type == MessageType.PHOTO


def test_depth_cap_requeue_into_empty_slot_is_preserved():
    """run.py depth-cap requeue (no merge_text): the event being requeued
    must land in the pending dict (the shadow dropped it entirely)."""
    pending = {}
    carried = _event("do the thing")
    base_mod.merge_pending_message_event(pending, "sk", carried)
    assert pending["sk"] is carried


def test_depth_cap_requeue_into_empty_slot_uses_helper_and_preserves():
    pending = {}
    carried = _event("do the thing")
    run_mod._requeue_carried_pending_event(pending, "sk", carried)
    assert pending["sk"] is carried


def test_depth_cap_requeue_with_newer_queued_text_keeps_both_in_order():
    """Older carried instruction goes first, newer queued text after it."""
    pending = {"sk": _event("newer instruction")}
    run_mod._requeue_carried_pending_event(pending, "sk", _event("earlier instruction"))
    assert pending["sk"].text == "earlier instruction\nnewer instruction"


def test_depth_cap_requeue_same_event_is_not_duplicated():
    carried = _event("only once")
    pending = {"sk": carried}
    run_mod._requeue_carried_pending_event(pending, "sk", carried)
    assert pending["sk"].text == "only once"


def test_depth_cap_requeue_media_into_queued_text_loses_nothing():
    pending = {"sk": _event("newer instruction")}
    carried = _event("see photo", MessageType.PHOTO, media=["a.jpg"])
    run_mod._requeue_carried_pending_event(pending, "sk", carried)
    merged = pending["sk"]
    assert merged.media_urls == ["a.jpg"]
    assert "newer instruction" in merged.text and "see photo" in merged.text


def test_bursty_followup_merge_text_true_appends_text():
    """Telegram burst follow-up (merge_text=True): text fragments append."""
    pending = {}
    base_mod.merge_pending_message_event(pending, "sk", _event("part one"), merge_text=True)
    base_mod.merge_pending_message_event(pending, "sk", _event("part two"), merge_text=True)
    assert pending["sk"].text == "part one\npart two"


def test_bursty_followup_without_merge_text_replaces():
    pending = {}
    base_mod.merge_pending_message_event(pending, "sk", _event("part one"))
    base_mod.merge_pending_message_event(pending, "sk", _event("part two"))
    assert pending["sk"].text == "part two"
