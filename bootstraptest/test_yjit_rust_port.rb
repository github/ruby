# Simple tests that we know we can pass
# To keep track of what we got working during the Rust port
# And avoid breaking/losing functionality

# Test for opt_mod
assert_equal '2', %q{
  def mod(a, b)
    a % b
  end

  mod(7, 5)
  mod(7, 5)
}

# Test for opt_mult
assert_equal '12', %q{
  def mult(a, b)
    a * b
  end

  mult(6, 2)
  mult(6, 2)
}

# Test for opt_div
assert_equal '3', %q{
  def div(a, b)
    a / b
  end

  div(6, 2)
  div(6, 2)
}

assert_equal '5', %q{
  def plus(a, b)
    a + b
  end

  plus(3, 2)
}

assert_equal '1', %q{
  def foo(a, b)
    a - b
  end

  foo(3, 2)
}

assert_equal 'true', %q{
  def foo(a, b)
    a < b
  end

  foo(2, 3)
}

# Bitwise left shift
assert_equal '4', %q{
  def foo(a, b)
    1 << 2
  end

  foo(1, 2)
}

assert_equal '-7', %q{
  def foo(a, b)
    -7
  end

  foo(1, 2)
}

assert_equal '-6', %q{
  def foo(a, b)
    a + -7
  end

  foo(1, 2)
}






# FIXME: currently takes the wrong branch
#
=begin
assert_equal '777', %q{
  def foo(a, b)
    if a
      777
    else
      333
    end
  end

  foo(true, true)
}
=end

# FIXME: loop never terminates, wrong branch direction
#
=begin
assert_equal '5', %q{
  def foo(a, b)
    while a < b
      a += 1
    end
    a
  end

  foo(1, 5)
}
=end

# FIXME: currently broken
# Assertion Failed: yjit.c:307:rb_iseq_pc_at_idx:insn_idx < iseq->body->iseq_size
#
=begin
# Function call
assert_equal '1', %q{
  def bar(a, b)
    a - b
  end

  def foo(a, b)
    bar(a, b)
  end

  foo(3, 2)
}
=end
