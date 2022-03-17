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

# Putstring
assert_equal 'foo', %q{
  def foo(a, b)
    "foo"
  end

  foo(1, 2)
}

assert_equal '-6', %q{
  def foo(a, b)
    a + -7
  end

  foo(1, 2)
}

assert_equal 'true', %q{
  def foo(a, b)
    a == b
  end

  foo(3, 3)
}

assert_equal 'true', %q{
  def foo(a, b)
    a < b
  end

  foo(3, 5)
}

assert_equal '777', %q{
  def foo(a)
    if a
      777
    else
      333
    end
  end

  foo(true)
}

assert_equal '5', %q{
  def foo(a, b)
    while a < b
      a += 1
    end
    a
  end

  foo(1, 5)
}

# opt_aref
assert_equal '2', %q{
  def foo(a, b)
    a[b]
  end

  foo([0, 1, 2], 2)
}

# Simple function calls with 0, 1, 2 arguments
assert_equal '-2', %q{
  def bar()
    -2
  end

  def foo(a, b)
    bar()
  end

  foo(3, 2)
}
assert_equal '2', %q{
  def bar(a)
    a
  end

  def foo(a, b)
    bar(b)
  end

  foo(3, 2)
}
assert_equal '1', %q{
  def bar(a, b)
    a - b
  end

  def foo(a, b)
    bar(a, b)
  end

  foo(3, 2)
}

# Regression test for assembler bug
assert_equal '1', %q{
  def check_index(index)
    if 0x40000000 < index
        return -1
    end
    1
  end

  check_index 2
}

# Setivar test
assert_equal '2', %q{
  class Klass
    attr_accessor :a

    def set()
        @a = 2
    end

    def get()
        @a
    end
  end

  o = Klass.new
  o.set()
  o.a
}

# Regression for putobject bug
assert_equal '1.5', %q{
  def foo(x)
    x
  end

  def bar
    foo(1.5)
  end

  bar()
}

# Getivar with an extended ivar table
assert_equal '3', %q{
  class Foo
    def initialize
      @x1 = 1
      @x2 = 1
      @x3 = 1
      @x4 = 3
    end

    def bar
      @x4
    end
  end

  f = Foo.new
  f.bar
}

assert_equal 'true', %q{
  x = [[false, true]]
  for i, j in x
    ;
  end
  j
}

# Regression for getivar
assert_equal '[nil]', %q{
  [TrueClass].each do |klass|
    klass.class_eval("def foo = @foo")
  end

  [true].map do |instance|
    instance.foo
  end
}

# Regression for send
assert_equal 'ok', %q{
  def bar(baz: 2)
    baz
  end

  def foo
    bar(1, baz: 123)
  end

  begin
    foo
    foo
  rescue ArgumentError => e
    print "ok"
  end
}

# Array access regression test
assert_equal '[0, 1, 2, 3, 4, 5]', %q{
  def expandarray_useless_splat
    arr = [0, 1, 2, 3, 4, 5]
    a, * = arr
  end

  expandarray_useless_splat
}

# Regression for clear_local_types()
assert_equal '2022', %q{
  def contrivance(hash, key)
    # Expect this to compile to an `opt_aref`.
    hash[key]

    # The [] call above tracks that the `hash` local has a VALUE that
    # is a heap pointer and the guard for the Kernel#itself call below
    # doesn't check that it's a heap pointer VALUE.
    #
    # As you can see from the crash, the call to rb_hash_aref() can set the
    # `hash` local, making eliding the heap object guard unsound.
    hash.itself
  end

  # This is similar to ->(recv, mid) { send(recv, mid).local_variable_set(...) }.
  # By composing we avoid creating new Ruby frames and so sending :binding
  # captures the environment of the frame that does the missing key lookup.
  # We use it to capture the environment inside of `contrivance`.
  cap_then_set =
    Kernel.instance_method(:send).method(:bind_call).to_proc >>
      ->(binding) { binding.local_variable_set(:hash, 2022) }
  special_missing = Hash.new(&cap_then_set)

  # Make YJIT speculate that it's a hash and generate code
  # that calls rb_hash_aref().
  contrivance({}, :warmup)

  contrivance(special_missing, :binding)
}

# Make sure we're correctly reading RStruct's as.ary union for embedded RStructs
assert_equal '3,12', %q{
  pt_struct = Struct.new(:x, :y)
  p = pt_struct.new(3, 12)
  def pt_inspect(pt)
    "#{pt.x},#{pt.y}"
  end

  # Make sure pt_inspect is JITted
  10.times { pt_inspect(p) }

  # Make sure it's returning '3,12' instead of e.g. '3,false'
  pt_inspect(p)
}
