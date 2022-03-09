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

# Test for opt_plus
assert_equal '5', %q{
  def plus(a, b)
    a + b
  end

  plus(3, 2)
}

assert_equal 'true', %q{
  def foo(a, b)
    a < b
  end

  foo(2, 3)
}

# FIXME: currently takes the wrong branch
#assert_equal '777', %q{
#  def foo(a, b)
#    if a
#      777
#    else
#      333
#    end
#  end
#
#  foo(true, true)
#}
