"""Step-3 tests: Registry — dup-id reject, canonical iteration, domain index."""

import dataclasses

import pytest
from hypothesis import given
from hypothesis import strategies as st

from simcore.environment import Environment
from simcore.flow import FlowResult
from simcore.ids import DomainId, FlowId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock


@dataclasses.dataclass(frozen=True)
class _NoOpFlow:
    """A structurally-valid Flow that evaluates to an empty (no-op) result."""

    id: FlowId
    priority: int = 0

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        return FlowResult(legs=())


def _flow(fid: str) -> _NoOpFlow:
    return _NoOpFlow(id=FlowId(fid))


def _stock(sid: str, domain: str) -> Stock:
    return Stock(
        id=StockId(sid),
        domain=DomainId(domain),
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=1.0,
        kind=StockKind.POOL,
    )


def test_registry_rejects_duplicate_flow_id() -> None:
    with pytest.raises(ValueError, match="duplicate"):
        Registry(flows=[_flow("f"), _flow("f")], stocks={})


def test_registry_iterates_in_canonical_id_order() -> None:
    reg = Registry(flows=[_flow("c"), _flow("a"), _flow("b")], stocks={})
    assert [flow.id for flow in reg] == ["a", "b", "c"]
    assert reg.flows == tuple(reg)  # the `.flows` view matches iteration
    assert len(reg) == 3


def test_registry_domain_index_matches_stock_domain() -> None:
    members = [
        _stock("bio.a", "bio"),
        _stock("bio.b", "bio"),
        _stock("bnd.x", "boundary"),
    ]
    reg = Registry(flows=[], stocks={s.id: s for s in members})
    assert reg.domain_index[DomainId("bio")] == frozenset(
        {StockId("bio.a"), StockId("bio.b")}
    )
    assert reg.domain_index[DomainId("boundary")] == frozenset({StockId("bnd.x")})
    assert set(reg.domain_index) == {DomainId("bio"), DomainId("boundary")}


def test_registry_domain_index_is_read_only() -> None:
    reg = Registry(flows=[], stocks={StockId("bio.a"): _stock("bio.a", "bio")})
    with pytest.raises(TypeError):
        reg.domain_index[DomainId("x")] = frozenset()  # type: ignore[index]


# Registration-order independence (#7/#15): any permutation of the same flows
# yields the *same* canonical (id-sorted) iteration. ASCII ids only, so Python's
# str sort matches the future Rust UTF-8 byte sort.
@given(
    ids=st.lists(
        st.text(
            alphabet="abcdefghijklmnopqrstuvwxyz0123456789._", min_size=1, max_size=8
        ),
        unique=True,
        min_size=1,
        max_size=12,
    ),
    data=st.data(),
)
def test_registry_registration_order_independence(
    ids: list[str], data: st.DataObject
) -> None:
    flows = [_flow(i) for i in ids]
    shuffled = data.draw(st.permutations(flows))
    reg = Registry(flows=shuffled, stocks={})
    assert [flow.id for flow in reg] == sorted(ids)
