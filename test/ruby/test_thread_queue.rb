# frozen_string_literal: false
require 'test/unit'
require 'tmpdir'
require 'timeout'

class TestThreadQueue < Test::Unit::TestCase
  Queue = Thread::Queue
  SizedQueue = Thread::SizedQueue

  def test_queue_initialized
    assert_raise_with_message(TypeError, /\bQueue.* not initialized/) {
      Queue.allocate.push(nil)
    }
  end

  def test_sized_queue_initialized
    assert_raise_with_message(TypeError, /\bSizedQueue.* not initialized/) {
      SizedQueue.allocate.push(nil)
    }
  end

  def test_freeze
    assert_raise(TypeError) {
      Queue.new.freeze
    }
    assert_raise(TypeError) {
      SizedQueue.new(5).freeze
    }
  end

  def test_queue
    grind(5, 1000, 15, Queue)
  end

  def test_sized_queue
    grind(5, 1000, 15, SizedQueue, 1000)
  end

  def grind(num_threads, num_objects, num_iterations, klass, *args)
    from_workers = klass.new(*args)
    to_workers = klass.new(*args)

    workers = (1..num_threads).map {
      Thread.new {
        while object = to_workers.pop
          from_workers.push object
        end
      }
    }

    Thread.new {
      num_iterations.times {
        num_objects.times { to_workers.push 99 }
        num_objects.times { from_workers.pop }
      }
    }.join

    # close the queue the old way to test for backwards-compatibility
    num_threads.times { to_workers.push nil }
    workers.each { |t| t.join }

    assert_equal 0, from_workers.size
    assert_equal 0, to_workers.size
  end

  def test_queue_initialize
    e = Class.new do
      include Enumerable
      def initialize(list) @list = list end
      def each(&block) @list.each(&block) end
    end

    all_assertions_foreach(nil,
                           [Array, "Array"],
                           [e, "Enumerable"],
                           [Struct.new(:to_a), "Array-like"],
                           ) do |a, type|
      q = Thread::Queue.new(a.new([1,2,3]))
      assert_equal(3, q.size, type)
      assert_not_predicate(q, :empty?, type)
      assert_equal(1, q.pop, type)
      assert_equal(2, q.pop, type)
      assert_equal(3, q.pop, type)
      assert_predicate(q, :empty?, type)
    end
  end

  def test_sized_queue_initialize
    q = Thread::SizedQueue.new(1)
    assert_equal 1, q.max
    assert_raise(ArgumentError) { Thread::SizedQueue.new(0) }
    assert_raise(ArgumentError) { Thread::SizedQueue.new(-1) }
  end

  def test_sized_queue_assign_max
    q = Thread::SizedQueue.new(2)
    assert_equal(2, q.max)
    q.max = 1
    assert_equal(1, q.max)
    assert_raise(ArgumentError) { q.max = 0 }
    assert_equal(1, q.max)
    assert_raise(ArgumentError) { q.max = -1 }
    assert_equal(1, q.max)

    before = q.max
    q.max.times { q << 1 }
    t1 = Thread.new { q << 1 }
    sleep 0.01 until t1.stop?
    q.max = q.max + 1
    assert_equal before + 1, q.max
  ensure
    t1.join if t1
  end

  def test_queue_pop_interrupt
    q = Thread::Queue.new
    t1 = Thread.new { q.pop }
    sleep 0.01 until t1.stop?
    t1.kill.join
    assert_equal(0, q.num_waiting)
  end

  def test_queue_pop_timeout
    q = Thread::Queue.new
    q << 1
    assert_equal 1, q.pop(timeout: 1)

    t1 = Thread.new { q.pop(timeout: 1) }
    assert_equal t1, t1.join(2)
    assert_nil t1.value

    t2 = Thread.new { q.pop(timeout: 0.1) }
    assert_equal t2, t2.join(1)
    assert_nil t2.value
  ensure
    t1&.kill&.join
    t2&.kill&.join
  end

  def test_queue_pop_non_block
    q = Thread::Queue.new
    assert_raise_with_message(ThreadError, /empty/) do
      q.pop(true)
    end
  end

  def test_sized_queue_pop_interrupt
    q = Thread::SizedQueue.new(1)
    t1 = Thread.new { q.pop }
    sleep 0.01 until t1.stop?
    t1.kill.join
    assert_equal(0, q.num_waiting)
  end

  def test_sized_queue_pop_timeout
    q = Thread::SizedQueue.new(1)

    q << 1
    assert_equal 1, q.pop(timeout: 1)

    t1 = Thread.new { q.pop(timeout: 1) }
    assert_equal t1, t1.join(2)
    assert_nil t1.value

    t2 = Thread.new { q.pop(timeout: 0.1) }
    assert_equal t2, t2.join(1)
    assert_nil t2.value
  ensure
    t1&.kill&.join
    t2&.kill&.join
  end

  def test_sized_queue_pop_non_block
    q = Thread::SizedQueue.new(1)
    assert_raise_with_message(ThreadError, /empty/) do
      q.pop(true)
    end
  end

  def test_sized_queue_push_timeout
    q = Thread::SizedQueue.new(1)

    q << 1
    assert_equal 1, q.size

    t1 = Thread.new { q.push(2, timeout: 1) }
    assert_equal t1, t1.join(2)
    assert_nil t1.value

    t2 = Thread.new { q.push(2, timeout: 0.1) }
    assert_equal t2, t2.join(1)
    assert_nil t2.value
  ensure
    t1&.kill&.join
    t2&.kill&.join
  end

  def test_sized_queue_push_interrupt
    q = Thread::SizedQueue.new(1)
    q.push(1)
    assert_raise_with_message(ThreadError, /full/) do
      q.push(2, true)
    end
  end

  def test_sized_queue_push_non_block
    q = Thread::SizedQueue.new(1)
    q.push(1)
    t1 = Thread.new { q.push(2) }
    sleep 0.01 until t1.stop?
    t1.kill.join
    assert_equal(0, q.num_waiting)
  end

  def test_thr_kill
    omit "[Bug #18613]" if /freebsd/ =~ RUBY_PLATFORM

    bug5343 = '[ruby-core:39634]'
    Dir.mktmpdir {|d|
      timeout = 60
      total_count = 250
      begin
        assert_normal_exit(<<-"_eom", bug5343, timeout: timeout, chdir: d)
          r, w = IO.pipe
          #{total_count}.times do |i|
            File.open("test_thr_kill_count", "w") {|f| f.puts i }
            queue = Thread::Queue.new
            th = Thread.start {
              queue.push(nil)
              r.read 1
            }
            queue.pop
            th.kill
            th.join
          end
        _eom
      rescue Timeout::Error
        # record load average:
        uptime = `uptime` rescue nil
        if uptime && /(load average: [\d.]+),/ =~ uptime
          la = " (#{$1})"
        end

        count = File.read("#{d}/test_thr_kill_count").to_i
        flunk "only #{count}/#{total_count} done in #{timeout} seconds.#{la}"
      end
    }
  end

  def test_queue_push_return_value
    q = Thread::Queue.new
    retval = q.push(1)
    assert_same q, retval
  end

  def test_queue_clear_return_value
    q = Thread::Queue.new
    retval = q.clear
    assert_same q, retval
  end

  def test_sized_queue_clear
    # Fill queue, then test that Thread::SizedQueue#clear wakes up all waiting threads
    sq = Thread::SizedQueue.new(2)
    2.times { sq << 1 }

    t1 = Thread.new do
      sq << 1
    end

    t2 = Thread.new do
      sq << 1
    end

    t3 = Thread.new do
      Thread.pass
      sq.clear
    end

    [t3, t2, t1].each(&:join)
    assert_equal sq.length, 2
  end

  def test_sized_queue_push_return_value
    q = Thread::SizedQueue.new(1)
    retval = q.push(1)
    assert_same q, retval
  end

  def test_sized_queue_clear_return_value
    q = Thread::SizedQueue.new(1)
    retval = q.clear
    assert_same q, retval
  end

  def test_sized_queue_throttle
    q = Thread::SizedQueue.new(1)
    i = 0
    consumer = Thread.new do
      while q.pop
        i += 1
        Thread.pass
      end
    end
    nprod = 4
    npush = 100

    producer = nprod.times.map do
      Thread.new do
        npush.times { q.push(true) }
      end
    end
    producer.each(&:join)
    q.push(nil)
    consumer.join
    assert_equal(nprod * npush, i)
  end

  def test_queue_thread_raise
    q = Thread::Queue.new
    th1 = Thread.new do
      begin
        q.pop
      rescue RuntimeError
        sleep
      end
    end
    th2 = Thread.new do
      sleep 0.1
      q.pop
    end
    sleep 0.1
    th1.raise
    sleep 0.1
    q << :s
    assert_nothing_raised(Timeout::Error) do
      Timeout.timeout(1) { th2.join }
    end
  ensure
    [th1, th2].each do |th|
      if th and th.alive?
        th.wakeup
        th.join
      end
    end
  end

  def test_dup
    bug9440 = '[ruby-core:59961] [Bug #9440]'
    q = Thread::Queue.new
    assert_raise(NoMethodError, bug9440) do
      q.dup
    end
  end

  (DumpableQueue = Queue.dup).class_eval {remove_method :marshal_dump}

  def test_dump
    bug9674 = '[ruby-core:61677] [Bug #9674]'
    q = Thread::Queue.new
    assert_raise_with_message(TypeError, /#{Queue}/, bug9674) do
      Marshal.dump(q)
    end

    sq = Thread::SizedQueue.new(1)
    assert_raise_with_message(TypeError, /#{SizedQueue}/, bug9674) do
      Marshal.dump(sq)
    end

    q = DumpableQueue.new
    assert_raise(TypeError, bug9674) do
      Marshal.dump(q)
    end
  end

  def test_close
    [->{Thread::Queue.new}, ->{Thread::SizedQueue.new 3}].each do |qcreate|
      q = qcreate.call
      assert_equal false, q.closed?
      q << :something
      assert_equal q, q.close
      assert q.closed?
      assert_raise_with_message(ClosedQueueError, /closed/){q << :nothing}
      assert_equal q.pop, :something
      assert_nil q.pop
      assert_nil q.pop
      # non-blocking
      assert_raise_with_message(ThreadError, /queue empty/){q.pop(non_block=true)}
    end
  end

  # test that waiting producers are woken up on close
  def close_wakeup( num_items, num_threads, &qcreate )
    raise "This test won't work with num_items(#{num_items}) >= num_threads(#{num_threads})" if num_items >= num_threads

    # create the Queue
    q = yield
    threads = num_threads.times.map{Thread.new{q.pop}}
    num_items.times{|i| q << i}

    # wait until queue empty
    (Thread.pass; sleep 0.01) until q.size == 0

    # close the queue so remaining threads will wake up
    q.close

    # wait for them to go away
    Thread.pass until threads.all?{|thr| thr.status == false}

    # check that they've gone away. Convert nil to -1 so we can sort and do the comparison
    expected_values = [-1] * (num_threads - num_items) + num_items.times.to_a
    assert_equal expected_values, threads.map{|thr| thr.value || -1 }.sort
  end

  def test_queue_close_wakeup
    close_wakeup(15, 18){Thread::Queue.new}
  end

  def test_size_queue_close_wakeup
    close_wakeup(5, 8){Thread::SizedQueue.new 9}
  end

  def test_sized_queue_one_closed_interrupt
    q = Thread::SizedQueue.new 1
    q << :one
    t1 = Thread.new {
      Thread.current.report_on_exception = false
      q << :two
    }
    sleep 0.01 until t1.stop?
    q.close
    assert_raise(ClosedQueueError) {t1.join}

    assert_equal 1, q.size
    assert_equal :one, q.pop
    assert q.empty?, "queue not empty"
  end

  # make sure that shutdown state is handled properly by empty? for the non-blocking case
  def test_empty_non_blocking
    q = Thread::SizedQueue.new 3
    3.times{|i| q << i}

    # these all block cos the queue is full
    prod_threads = 4.times.map {|i|
      Thread.new {
        Thread.current.report_on_exception = false
        q << 3+i
      }
    }
    sleep 0.01 until prod_threads.all?{|thr| thr.stop?}

    items = []
    # sometimes empty? is false but pop will raise ThreadError('empty'),
    # meaning a value is not immediately available but will be soon.
    while prod_threads.any?(&:alive?) or !q.empty?
      items << q.pop(true) rescue nil
    end
    assert_join_threads(prod_threads)
    items.compact!

    assert_equal 7.times.to_a, items.sort
    assert q.empty?
  end

  def test_sized_queue_closed_push_non_blocking
    q = Thread::SizedQueue.new 7
    q.close
    assert_raise_with_message(ClosedQueueError, /queue closed/){q.push(non_block=true)}
  end

  def test_blocked_pushers
    q = Thread::SizedQueue.new 3
    prod_threads = 6.times.map do |i|
      thr = Thread.new{
        Thread.current.report_on_exception = false
        q << i
      }
      thr[:pc] = i
      thr
    end

    # wait until some producer threads have finished, and the other 3 are blocked
    sleep 0.01 while prod_threads.reject{|t| t.status}.count < 3
    # this would ensure that all producer threads call push before close
    # sleep 0.01 while prod_threads.select{|t| t.status == 'sleep'}.count < 3
    q.close

    # more than prod_threads
    cons_threads = 10.times.map do |i|
      thr = Thread.new{q.pop}; thr[:pc] = i; thr
    end

    # values that came from the queue
    popped_values = cons_threads.map &:value

    # wait untl all threads have finished
    sleep 0.01 until prod_threads.find_all{|t| t.status}.count == 0

    # pick only the producer threads that got in before close
    successful_prod_threads = prod_threads.reject{|thr| thr.status == nil}
    assert_nothing_raised{ successful_prod_threads.map(&:value) }

    # the producer threads that tried to push after q.close should all fail
    unsuccessful_prod_threads = prod_threads - successful_prod_threads
    unsuccessful_prod_threads.each do |thr|
      assert_raise(ClosedQueueError){ thr.value }
    end

    assert_equal cons_threads.size, popped_values.size
    assert_equal 0, q.size

    # check that consumer threads with values match producers that called push before close
    assert_equal successful_prod_threads.map{|thr| thr[:pc]}, popped_values.compact.sort
    assert_nil q.pop
  end

  def test_deny_pushers
    [->{Thread::Queue.new}, ->{Thread::SizedQueue.new 3}].each do |qcreate|
      q = qcreate[]
      synq = Thread::Queue.new
      prod_threads = 20.times.map do |i|
        Thread.new {
          synq.pop
          assert_raise(ClosedQueueError) {
            q << i
          }
        }
      end
      q.close
      synq.close # start producer threads

      prod_threads.each(&:join)
    end
  end

  # size should account for waiting pushers during shutdown
  def sized_queue_size_close
    q = Thread::SizedQueue.new 4
    4.times{|i| q << i}
    Thread.new{ q << 5 }
    Thread.new{ q << 6 }
    assert_equal 4, q.size
    assert_equal 4, q.items
    q.close
    assert_equal 6, q.size
    assert_equal 4, q.items
  end

  def test_blocked_pushers_empty
    q = Thread::SizedQueue.new 3
    prod_threads = 6.times.map do |i|
      Thread.new{
        Thread.current.report_on_exception = false
        q << i
      }
    end

    # this ensures that all producer threads call push before close
    sleep 0.01 while prod_threads.select{|t| t.status == 'sleep'}.count < 3
    q.close

    ary = []
    until q.empty?
      ary << q.pop
    end
    assert_equal 0, q.size

    assert_equal 3, ary.size
    ary.each{|e| assert [0,1,2,3,4,5].include?(e)}
    assert_nil q.pop

    prod_threads.each{|t|
      begin
        t.join
      rescue
      end
    }
  end

  # test thread wakeup on one-element SizedQueue with close
  def test_one_element_sized_queue
    q = Thread::SizedQueue.new 1
    t = Thread.new{ q.pop }
    q.close
    assert_nil t.value
  end

  def test_close_twice
    [->{Thread::Queue.new}, ->{Thread::SizedQueue.new 3}].each do |qcreate|
      q = qcreate[]
      q.close
      assert_nothing_raised(ClosedQueueError){q.close}
    end
  end

  def test_queue_close_multi_multi
    q = Thread::SizedQueue.new rand(800..1200)

    count_items = rand(3000..5000)
    count_producers = rand(10..20)

    # ensure threads do not start running too soon and complete before we check status
    mutex = Mutex.new
    mutex.lock

    producers = count_producers.times.map do
      Thread.new do
        mutex.lock
        mutex.unlock
        count_items.times{|i| q << [i,"#{i} for #{Thread.current.inspect}"]}
      end
    end

    consumers = rand(7..12).times.map do
      Thread.new do
        count = 0
        while e = q.pop
          i, st = e
          count += 1 if i.is_a?(Integer) && st.is_a?(String)
        end
        count
      end
    end

    # No dead or finished threads, give up to 10 seconds to start running
    t = Time.now
    Thread.pass until Time.now - t > 10 || (consumers + producers).all?{|thr| thr.status.to_s =~ /\A(?:run|sleep)\z/}

    assert (consumers + producers).all?{|thr| thr.status.to_s =~ /\A(?:run|sleep)\z/}, 'no threads running'

    mutex.unlock

    # just exercising the concurrency of the support methods.
    counter = Thread.new do
      until q.closed? && q.empty?
        raise if q.size > q.max
        # otherwise this exercise causes too much contention on the lock
        sleep 0.01
      end
    end

    producers.each &:join
    q.close

    # results not randomly distributed. Not sure why.
    # consumers.map{|thr| thr.value}.each do |x|
    #   assert_not_equal 0, x
    # end

    all_items_count = consumers.map{|thr| thr.value}.inject(:+)
    assert_equal count_items * count_producers, all_items_count

    # don't leak this thread
    assert_nothing_raised{counter.join}
  end

  def test_queue_with_trap
    if ENV['APPVEYOR'] == 'True' && RUBY_PLATFORM.match?(/mswin/)
      omit 'This test fails too often on AppVeyor vs140'
    end
    if RUBY_PLATFORM.match?(/mingw/)
      omit 'This test fails too often on MinGW'
    end

    assert_in_out_err([], <<-INPUT, %w(INT INT exit), [])
      q = Thread::Queue.new
      trap(:INT){
        q.push 'INT'
      }
      Thread.new{
        loop{
          Process.kill :INT, $$
          sleep 0.1
        }
      }
      puts q.pop
      puts q.pop
      puts 'exit'
    INPUT
  end

  def test_fork_while_queue_waiting
    q = Thread::Queue.new
    sq = Thread::SizedQueue.new(1)
    thq = Thread.new { q.pop }
    thsq = Thread.new { sq.pop }
    Thread.pass until thq.stop? && thsq.stop?

    pid = fork do
      exit!(1) if q.num_waiting != 0
      exit!(2) if sq.num_waiting != 0
      exit!(6) unless q.empty?
      exit!(7) unless sq.empty?
      q.push :child_q
      sq.push :child_sq
      exit!(3) if q.pop != :child_q
      exit!(4) if sq.pop != :child_sq
      exit!(0)
    end
    _, s = Process.waitpid2(pid)
    assert_predicate s, :success?, 'no segfault [ruby-core:86316] [Bug #14634]'

    q.push :thq
    sq.push :thsq
    assert_equal :thq, thq.value
    assert_equal :thsq, thsq.value

    sq.push(1)
    th = Thread.new { q.pop; sq.pop }
    thsq = Thread.new { sq.push(2) }
    Thread.pass until th.stop? && thsq.stop?
    pid = fork do
      exit!(1) if q.num_waiting != 0
      exit!(2) if sq.num_waiting != 0
      exit!(3) unless q.empty?
      exit!(4) if sq.empty?
      exit!(5) if sq.pop != 1
      exit!(0)
    end
    _, s = Process.waitpid2(pid)
    assert_predicate s, :success?, 'no segfault [ruby-core:86316] [Bug #14634]'

    assert_predicate thsq, :stop?
    assert_equal 1, sq.pop
    assert_same sq, thsq.value
    q.push('restart th')
    assert_equal 2, th.value
  end if Process.respond_to?(:fork)
end
