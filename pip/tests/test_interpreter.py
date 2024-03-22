# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

from textwrap import dedent
from qsharp._native import (
    Interpreter,
    Result,
    Pauli,
    QSharpError,
    TargetProfile,
)
import pytest


# Tests for the native Q# interpreter class


def test_output() -> None:
    e = Interpreter(TargetProfile.Unrestricted)

    def callback(output):
        nonlocal called
        called = True
        assert output.__repr__() == "Hello, world!"

    called = False
    value = e.interpret('Message("Hello, world!")', callback)
    assert called


def test_dump_output() -> None:
    e = Interpreter(TargetProfile.Unrestricted)

    def callback(output):
        nonlocal called
        called = True
        assert output.__repr__() == "STATE:\n|10⟩: 1.0000+0.0000𝑖"

    called = False
    value = e.interpret(
        """
    use q1 = Qubit();
    use q2 = Qubit();
    X(q1);
    Microsoft.Quantum.Diagnostics.DumpMachine();
    ResetAll([q1, q2]);
    """,
        callback,
    )
    assert called


def test_quantum_seed() -> None:
    e = Interpreter(TargetProfile.Unrestricted)
    e.set_quantum_seed(42)
    value1 = e.interpret(
        "{ use qs = Qubit[16]; for q in qs { H(q); }; Microsoft.Quantum.Measurement.MResetEachZ(qs) }"
    )
    e = Interpreter(TargetProfile.Unrestricted)
    e.set_quantum_seed(42)
    value2 = e.interpret(
        "{ use qs = Qubit[16]; for q in qs { H(q); }; Microsoft.Quantum.Measurement.MResetEachZ(qs) }"
    )
    assert value1 == value2


def test_classical_seed() -> None:
    e = Interpreter(TargetProfile.Unrestricted)
    e.set_classical_seed(42)
    value1 = e.interpret(
        "{ mutable res = []; for _ in 0..15{ set res += [Microsoft.Quantum.Random.DrawRandomInt(0, 100)]; }; res }"
    )
    e = Interpreter(TargetProfile.Unrestricted)
    e.set_classical_seed(42)
    value2 = e.interpret(
        "{ mutable res = []; for _ in 0..15{ set res += [Microsoft.Quantum.Random.DrawRandomInt(0, 100)]; }; res }"
    )
    assert value1 == value2


def test_dump_machine() -> None:
    e = Interpreter(TargetProfile.Unrestricted)

    def callback(output):
        assert output.__repr__() == "STATE:\n|10⟩: 1.0000+0.0000𝑖"

    value = e.interpret(
        """
    use q1 = Qubit();
    use q2 = Qubit();
    X(q1);
    Microsoft.Quantum.Diagnostics.DumpMachine();
    """,
        callback,
    )
    state_dump = e.dump_machine()
    assert state_dump.qubit_count == 2
    state_dump = state_dump.get_dict()
    assert len(state_dump) == 1
    assert state_dump[2].real == 1.0
    assert state_dump[2].imag == 0.0


def test_error() -> None:
    e = Interpreter(TargetProfile.Unrestricted)

    with pytest.raises(QSharpError) as excinfo:
        e.interpret("a864")
    assert str(excinfo.value).find("name error") != -1


def test_multiple_errors() -> None:
    e = Interpreter(TargetProfile.Unrestricted)

    with pytest.raises(QSharpError) as excinfo:
        e.interpret("operation Foo() : Unit { Bar(); Baz(); }")
    assert str(excinfo.value).find("`Bar` not found") != -1
    assert str(excinfo.value).find("`Baz` not found") != -1


def test_multiple_statements() -> None:
    e = Interpreter(TargetProfile.Unrestricted)
    value = e.interpret("1; Zero")
    assert value == Result.Zero


def test_value_int() -> None:
    e = Interpreter(TargetProfile.Unrestricted)
    value = e.interpret("5")
    assert value == 5


def test_value_double() -> None:
    e = Interpreter(TargetProfile.Unrestricted)
    value = e.interpret("3.1")
    assert value == 3.1


def test_value_bool() -> None:
    e = Interpreter(TargetProfile.Unrestricted)
    value = e.interpret("true")
    assert value == True


def test_value_string() -> None:
    e = Interpreter(TargetProfile.Unrestricted)
    value = e.interpret('"hello"')
    assert value == "hello"


def test_value_result() -> None:
    e = Interpreter(TargetProfile.Unrestricted)
    value = e.interpret("One")
    assert value == Result.One


def test_value_pauli() -> None:
    e = Interpreter(TargetProfile.Unrestricted)
    value = e.interpret("PauliX")
    assert value == Pauli.X


def test_value_tuple() -> None:
    e = Interpreter(TargetProfile.Unrestricted)
    value = e.interpret('(1, "hello", One)')
    assert value == (1, "hello", Result.One)


def test_value_unit() -> None:
    e = Interpreter(TargetProfile.Unrestricted)
    value = e.interpret("()")
    assert value is None


def test_value_array() -> None:
    e = Interpreter(TargetProfile.Unrestricted)
    value = e.interpret("[1, 2, 3]")
    assert value == [1, 2, 3]


def test_target_error() -> None:
    e = Interpreter(TargetProfile.Base)
    with pytest.raises(QSharpError) as excinfo:
        e.interpret("operation Program() : Result { return Zero }")
    assert str(excinfo.value).startswith("Qsc.BaseProfCk.ResultLiteral") != -1


def test_qirgen_compile_error() -> None:
    e = Interpreter(TargetProfile.Base)
    e.interpret("operation Program() : Int { return 0 }")
    with pytest.raises(QSharpError) as excinfo:
        e.qir("Foo()")
    assert str(excinfo.value).startswith("Qsc.Resolve.NotFound") != -1


def test_error_spans_from_multiple_lines() -> None:
    e = Interpreter(TargetProfile.Unrestricted)

    # Qsc.Resolve.Ambiguous is chosen as a test case
    # because it contains multiple spans which can be from different lines
    e.interpret("namespace Other { operation DumpMachine() : Unit { } }")
    e.interpret("open Other;")
    e.interpret("open Microsoft.Quantum.Diagnostics;")
    with pytest.raises(QSharpError) as excinfo:
        e.interpret("DumpMachine()")
    assert str(excinfo.value).startswith("Qsc.Resolve.Ambiguous")


def test_qirgen() -> None:
    e = Interpreter(TargetProfile.Base)
    e.interpret("operation Program() : Result { use q = Qubit(); return M(q) }")
    qir = e.qir("Program()")
    assert isinstance(qir, str)


def test_run_with_shots() -> None:
    e = Interpreter(TargetProfile.Unrestricted)

    def callback(output):
        nonlocal called
        called += 1
        assert output.__repr__() == "Hello, world!"

    called = 0
    e.interpret('operation Foo() : Unit { Message("Hello, world!"); }', callback)
    assert called == 0

    value = []
    for _ in range(5):
        value.append(e.run("Foo()", callback))
    assert called == 5

    assert value == [None, None, None, None, None]


def test_dump_circuit() -> None:
    e = Interpreter(TargetProfile.Unrestricted)
    e.interpret(
        """
    use q1 = Qubit();
    use q2 = Qubit();
    X(q1);
    """
    )
    circuit = e.dump_circuit()
    assert str(circuit) == dedent(
        """\
        q_0    ── X ──
        q_1    ───────
        """
    )

    e.interpret("X(q2);")
    circuit = e.dump_circuit()
    assert str(circuit) == dedent(
        """\
        q_0    ── X ──
        q_1    ── X ──
        """
    )


def test_entry_expr_circuit() -> None:
    e = Interpreter(TargetProfile.Unrestricted)
    e.interpret("operation Foo() : Result { use q = Qubit(); H(q); return M(q) }")
    circuit = e.circuit("Foo()")
    assert str(circuit) == dedent(
        """\
        q_0    ── H ──── M ──
                         ╘═══
        """
    )


def test_operation_circuit() -> None:
    e = Interpreter(TargetProfile.Unrestricted)
    e.interpret("operation Foo(q: Qubit) : Result { H(q); return M(q) }")
    circuit = e.circuit(operation="Foo")
    assert str(circuit) == dedent(
        """\
        q_0    ── H ──── M ──
                         ╘═══
        """
    )


def test_unsupported_operation_circuit() -> None:
    e = Interpreter(TargetProfile.Unrestricted)
    e.interpret("operation Foo(n: Int) : Result { return One }")
    with pytest.raises(QSharpError) as excinfo:
        circuit = e.circuit(operation="Foo")
    assert (
        str(excinfo.value).find(
            "expression does not evaluate to an operation that takes qubit parameters"
        )
        != -1
    )
