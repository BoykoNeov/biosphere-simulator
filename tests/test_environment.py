"""Step-3 tests: the Environment Protocol (interface only; backends are step 5)."""

from simcore.environment import Environment


def test_environment_protocol_accepts_a_get_implementation() -> None:
    class Forcing:
        def get(self, var: str) -> float:
            return 1.5

    assert isinstance(Forcing(), Environment)


def test_environment_protocol_rejects_missing_get() -> None:
    class NotAnEnv:
        def lookup(self, var: str) -> float:
            return 0.0

    assert not isinstance(NotAnEnv(), Environment)


def test_environment_get_returns_the_value() -> None:
    class Forcing:
        def get(self, var: str) -> float:
            return 2.0

    env: Environment = Forcing()
    assert env.get("light") == 2.0
