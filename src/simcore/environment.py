"""The Environment source-resolver Protocol (interface only).

Phase-0 step 3 builds **only** this interface — the concrete forcing-schedule and
shared-stock backends are step 5. ``Flow.evaluate`` needs the type in its
signature now; building the backends here would drag step 5 forward.

A flow calls ``env.get(var)`` and **cannot tell** whether the value came from a
forcing schedule (evaluated at ``t = n*dt``, integer ``n``) or a sibling domain's
shared stock (read from the *same* immutable snapshot the flows read, decision
#16). That indistinguishability is the point: identical domain code runs both
standalone and coupled.

Pure stdlib only.
"""

from typing import Protocol, runtime_checkable


@runtime_checkable
class Environment(Protocol):
    """Resolves an environment variable name to a scalar value.

    The caller cannot distinguish a forcing-schedule source from a shared-stock
    source — both are just ``get(var) -> float`` (decision #16).
    """

    def get(self, var: str) -> float: ...
