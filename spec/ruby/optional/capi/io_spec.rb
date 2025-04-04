require_relative 'spec_helper'
require_relative '../../fixtures/io'

load_extension('io')

describe "C-API IO function" do
  before :each do
    @o = CApiIOSpecs.new

    @name = tmp("c_api_rb_io_specs")
    touch @name

    @io = new_io @name, "w:utf-8"
    @io.sync = true
  end

  after :each do
    @io.close unless @io.closed?
    rm_r @name
  end

  describe "rb_io_addstr" do
    it "calls #to_s to convert the object to a String" do
      obj = mock("rb_io_addstr string")
      obj.should_receive(:to_s).and_return("rb_io_addstr data")

      @o.rb_io_addstr(@io, obj)
      File.read(@name).should == "rb_io_addstr data"
    end

    it "writes the String to the IO" do
      @o.rb_io_addstr(@io, "rb_io_addstr data")
      File.read(@name).should == "rb_io_addstr data"
    end

    it "returns the io" do
      @o.rb_io_addstr(@io, "rb_io_addstr data").should eql(@io)
    end
  end

  describe "rb_io_printf" do
    it "calls #to_str to convert the format object to a String" do
      obj = mock("rb_io_printf format")
      obj.should_receive(:to_str).and_return("%s")

      @o.rb_io_printf(@io, [obj, "rb_io_printf"])
      File.read(@name).should == "rb_io_printf"
    end

    it "calls #to_s to convert the object to a String" do
      obj = mock("rb_io_printf string")
      obj.should_receive(:to_s).and_return("rb_io_printf")

      @o.rb_io_printf(@io, ["%s", obj])
      File.read(@name).should == "rb_io_printf"
    end

    it "writes the Strings to the IO" do
      @o.rb_io_printf(@io, ["%s_%s_%s", "rb", "io", "printf"])
      File.read(@name).should == "rb_io_printf"
    end
  end

  describe "rb_io_print" do
    it "calls #to_s to convert the object to a String" do
      obj = mock("rb_io_print string")
      obj.should_receive(:to_s).and_return("rb_io_print")

      @o.rb_io_print(@io, [obj])
      File.read(@name).should == "rb_io_print"
    end

    it "writes the Strings to the IO with no separator" do
      @o.rb_io_print(@io, ["rb_", "io_", "print"])
      File.read(@name).should == "rb_io_print"
    end
  end

  describe "rb_io_puts" do
    it "calls #to_s to convert the object to a String" do
      obj = mock("rb_io_puts string")
      obj.should_receive(:to_s).and_return("rb_io_puts")

      @o.rb_io_puts(@io, [obj])
      File.read(@name).should == "rb_io_puts\n"
    end

    it "writes the Strings to the IO separated by newlines" do
      @o.rb_io_puts(@io, ["rb", "io", "write"])
      File.read(@name).should == "rb\nio\nwrite\n"
    end
  end

  describe "rb_io_write" do
    it "calls #to_s to convert the object to a String" do
      obj = mock("rb_io_write string")
      obj.should_receive(:to_s).and_return("rb_io_write")

      @o.rb_io_write(@io, obj)
      File.read(@name).should == "rb_io_write"
    end

    it "writes the String to the IO" do
      @o.rb_io_write(@io, "rb_io_write")
      File.read(@name).should == "rb_io_write"
    end
  end
end

describe "C-API IO function" do
  before :each do
    @o = CApiIOSpecs.new

    @name = tmp("c_api_io_specs")
    touch @name

    @io = new_io @name, "r:utf-8"
  end

  after :each do
    @io.close unless @io.closed?
    rm_r @name
  end

  describe "rb_io_close" do
    it "closes an IO object" do
      @io.closed?.should be_false
      @o.rb_io_close(@io)
      @io.closed?.should be_true
    end
  end

  describe "rb_io_check_io" do
    it "returns the IO object if it is valid" do
      @o.rb_io_check_io(@io).should == @io
    end

    it "returns nil for non IO objects" do
      @o.rb_io_check_io({}).should be_nil
    end
  end

  describe "rb_io_check_closed" do
    it "does not raise an exception if the IO is not closed" do
      # The MRI function is void, so we use should_not raise_error
      -> { @o.rb_io_check_closed(@io) }.should_not raise_error
    end

    it "raises an error if the IO is closed" do
      @io.close
      -> { @o.rb_io_check_closed(@io) }.should raise_error(IOError)
    end
  end

  describe "rb_io_set_nonblock" do
    platform_is_not :windows do
      it "returns true when nonblock flag is set" do
        require 'io/nonblock'
        @o.rb_io_set_nonblock(@io)
        @io.nonblock?.should be_true
      end
    end
  end

  # NOTE: unlike the name might suggest in MRI this function checks if an
  # object is frozen, *not* if it's tainted.
  describe "rb_io_taint_check" do
    it "does not raise an exception if the IO is not frozen" do
      -> { @o.rb_io_taint_check(@io) }.should_not raise_error
    end

    it "raises an exception if the IO is frozen" do
      @io.freeze

      -> { @o.rb_io_taint_check(@io) }.should raise_error(RuntimeError)
    end
  end

  describe "rb_io_descriptor or GetOpenFile" do
    it "allows access to the system fileno" do
      @o.GetOpenFile_fd($stdin).should == 0
      @o.GetOpenFile_fd($stdout).should == 1
      @o.GetOpenFile_fd($stderr).should == 2
      @o.GetOpenFile_fd(@io).should == @io.fileno
    end

    it "raises IOError if the IO is closed" do
      @io.close
      -> { @o.GetOpenFile_fd(@io) }.should raise_error(IOError, "closed stream")
    end
  end

  describe "rb_io_binmode" do
    it "returns self" do
      @o.rb_io_binmode(@io).should == @io
    end

    it "sets binmode" do
      @o.rb_io_binmode(@io)
      @io.binmode?.should be_true
    end
  end
end

describe "C-API IO function" do
  before :each do
    @o = CApiIOSpecs.new
    @r_io, @w_io = IO.pipe

    @name = tmp("c_api_io_specs")
    touch @name
    @rw_io = new_io @name, "w+"
  end

  after :each do
    @r_io.close unless @r_io.closed?
    @w_io.close unless @w_io.closed?
    @rw_io.close unless @rw_io.closed?
    rm_r @name
  end

  describe "rb_io_check_readable" do
    it "does not raise an exception if the IO is opened for reading" do
      # The MRI function is void, so we use should_not raise_error
      -> { @o.rb_io_check_readable(@r_io) }.should_not raise_error
    end

    it "does not raise an exception if the IO is opened for read and write" do
      -> { @o.rb_io_check_readable(@rw_io) }.should_not raise_error
    end

    it "raises an IOError if the IO is not opened for reading" do
      -> { @o.rb_io_check_readable(@w_io) }.should raise_error(IOError)
    end

  end

  describe "rb_io_check_writable" do
    it "does not raise an exception if the IO is opened for writing" do
      # The MRI function is void, so we use should_not raise_error
      -> { @o.rb_io_check_writable(@w_io) }.should_not raise_error
    end

    it "does not raise an exception if the IO is opened for read and write" do
      -> { @o.rb_io_check_writable(@rw_io) }.should_not raise_error
    end

    it "raises an IOError if the IO is not opened for reading" do
      -> { @o.rb_io_check_writable(@r_io) }.should raise_error(IOError)
    end
  end

  describe "rb_io_wait_writable" do
    it "returns false if there is no error condition" do
      @o.errno = 0
      @o.rb_io_wait_writable(@w_io).should be_false
    end

    it "raises an IOError if the IO is closed" do
      @w_io.close
      -> { @o.rb_io_wait_writable(@w_io) }.should raise_error(IOError)
    end
  end

  describe "rb_io_maybe_wait_writable" do
    it "returns mask for events if operation was interrupted" do
      @o.rb_io_maybe_wait_writable(Errno::EINTR::Errno, @w_io, nil).should == IO::WRITABLE
    end

    it "returns 0 if there is no error condition" do
      @o.rb_io_maybe_wait_writable(0, @w_io, nil).should == 0
    end

    it "raises an IOError if the IO is closed" do
      @w_io.close
      -> { @o.rb_io_maybe_wait_writable(0, @w_io, nil) }.should raise_error(IOError, "closed stream")
    end

    it "raises an IOError if the IO is not initialized" do
      -> { @o.rb_io_maybe_wait_writable(0, IO.allocate, nil) }.should raise_error(IOError, "uninitialized stream")
    end

    it "can be interrupted" do
      IOSpec.exhaust_write_buffer(@w_io)
      start = Process.clock_gettime(Process::CLOCK_MONOTONIC)

      t = Thread.new do
        @o.rb_io_maybe_wait_writable(0, @w_io, 10)
      end

      Thread.pass until t.stop?
      t.kill
      t.join

      finish = Process.clock_gettime(Process::CLOCK_MONOTONIC)
      (finish - start).should < 9
    end
  end

  describe "rb_thread_fd_writable" do
    it "waits til an fd is ready for writing" do
      @o.rb_thread_fd_writable(@w_io).should be_nil
    end
  end

  describe "rb_thread_fd_select" do
    it "waits until an fd is ready for reading" do
      @w_io.write "rb_thread_fd_select"
      @o.rb_thread_fd_select_read(@r_io).should == 1
    end

    it "waits until an fd is ready for writing" do
      @o.rb_thread_fd_select_write(@w_io).should == 1
    end

    it "waits until an fd is ready for writing with timeout" do
      @o.rb_thread_fd_select_timeout(@w_io).should == 1
    end
  end

  platform_is_not :windows do
    describe "rb_io_wait_readable" do
      it "returns false if there is no error condition" do
        @o.errno = 0
        @o.rb_io_wait_readable(@r_io, false).should be_false
      end

      it "raises and IOError if passed a closed stream" do
        @r_io.close
        -> {
          @o.rb_io_wait_readable(@r_io, false)
        }.should raise_error(IOError)
      end

      it "blocks until the io is readable and returns true" do
        @o.instance_variable_set :@write_data, false
        thr = Thread.new do
          Thread.pass until @o.instance_variable_get(:@write_data)
          @w_io.write "rb_io_wait_readable"
        end

        @o.errno = Errno::EAGAIN.new.errno
        @o.rb_io_wait_readable(@r_io, true).should be_true
        @o.instance_variable_get(:@read_data).should == "rb_io_wait_re"

        thr.join
      end
    end

    describe "rb_io_maybe_wait_readable" do
      it "returns mask for events if operation was interrupted" do
        @o.rb_io_maybe_wait_readable(Errno::EINTR::Errno, @r_io, nil, false).should == IO::READABLE
      end

      it "returns 0 if there is no error condition" do
        @o.rb_io_maybe_wait_readable(0, @r_io, nil, false).should == 0
      end

      it "blocks until the io is readable and returns events that actually occurred" do
        @o.instance_variable_set :@write_data, false
        thr = Thread.new do
          Thread.pass until @o.instance_variable_get(:@write_data)
          @w_io.write "rb_io_wait_readable"
        end

        @o.rb_io_maybe_wait_readable(Errno::EAGAIN::Errno, @r_io, IO::READABLE, true).should == IO::READABLE
        @o.instance_variable_get(:@read_data).should == "rb_io_wait_re"

        thr.join
      end

      it "can be interrupted" do
        start = Process.clock_gettime(Process::CLOCK_MONOTONIC)

        t = Thread.new do
          @o.rb_io_maybe_wait_readable(0, @r_io, 10, false)
        end

        Thread.pass until t.stop?
        t.kill
        t.join

        finish = Process.clock_gettime(Process::CLOCK_MONOTONIC)
        (finish - start).should < 9
      end

      it "raises an IOError if the IO is closed" do
        @r_io.close
        -> { @o.rb_io_maybe_wait_readable(0, @r_io, nil, false) }.should raise_error(IOError, "closed stream")
      end

      it "raises an IOError if the IO is not initialized" do
        -> { @o.rb_io_maybe_wait_readable(0, IO.allocate, nil, false) }.should raise_error(IOError, "uninitialized stream")
      end
    end
  end

  describe "rb_thread_wait_fd" do
    it "waits til an fd is ready for reading" do
      start = false
      thr = Thread.new do
        start = true
        sleep 0.05
        @w_io.write "rb_io_wait_readable"
      end

      Thread.pass until start

      @o.rb_thread_wait_fd(@r_io).should be_nil

      thr.join
    end
  end

  describe "rb_wait_for_single_fd" do
    it "waits til an fd is ready for reading" do
      start = false
      thr = Thread.new do
        start = true
        sleep 0.05
        @w_io.write "rb_io_wait_readable"
      end

      Thread.pass until start

      @o.rb_wait_for_single_fd(@r_io, 1, nil, nil).should == 1

      thr.join
    end

    it "polls whether an fd is ready for reading if timeout is 0" do
      @o.rb_wait_for_single_fd(@r_io, 1, 0, 0).should == 0
    end
  end

  describe "rb_io_maybe_wait" do
    it "waits til an fd is ready for reading" do
      start = false
      thr = Thread.new do
        start = true
        sleep 0.05
        @w_io.write "rb_io_maybe_wait"
      end

      Thread.pass until start

      @o.rb_io_maybe_wait(Errno::EAGAIN::Errno, @r_io, IO::READABLE, nil).should == IO::READABLE

      thr.join
    end

    it "returns mask for events if operation was interrupted" do
      @o.rb_io_maybe_wait(Errno::EINTR::Errno, @w_io, IO::WRITABLE, nil).should == IO::WRITABLE
    end

    it "raises an IOError if the IO is closed" do
      @w_io.close
      -> { @o.rb_io_maybe_wait(0, @w_io, IO::WRITABLE, nil) }.should raise_error(IOError, "closed stream")
    end

    it "raises an IOError if the IO is not initialized" do
      -> { @o.rb_io_maybe_wait(0, IO.allocate, IO::WRITABLE, nil) }.should raise_error(IOError, "uninitialized stream")
    end

    it "can be interrupted when waiting for READABLE event" do
      start = Process.clock_gettime(Process::CLOCK_MONOTONIC)

      t = Thread.new do
        @o.rb_io_maybe_wait(0, @r_io, IO::READABLE, 10)
      end

      Thread.pass until t.stop?
      t.kill
      t.join

      finish = Process.clock_gettime(Process::CLOCK_MONOTONIC)
      (finish - start).should < 9
    end

    it "can be interrupted when waiting for WRITABLE event" do
      IOSpec.exhaust_write_buffer(@w_io)
      start = Process.clock_gettime(Process::CLOCK_MONOTONIC)

      t = Thread.new do
        @o.rb_io_maybe_wait(0, @w_io, IO::WRITABLE, 10)
      end

      Thread.pass until t.stop?
      t.kill
      t.join

      finish = Process.clock_gettime(Process::CLOCK_MONOTONIC)
      (finish - start).should < 9
    end
  end

  ruby_version_is "3.3" do
    describe "rb_io_mode" do
      it "returns the mode" do
        (@o.rb_io_mode(@r_io) & 0b11).should == 0b01
        (@o.rb_io_mode(@w_io) & 0b11).should == 0b10
        (@o.rb_io_mode(@rw_io) & 0b11).should == 0b11
      end
    end

    describe "rb_io_path" do
      it "returns the IO#path" do
        @o.rb_io_path(@r_io).should == @r_io.path
        @o.rb_io_path(@rw_io).should == @rw_io.path
        @o.rb_io_path(@rw_io).should == @name
      end
    end

    describe "rb_io_closed_p" do
      it "returns false when io is not closed" do
        @o.rb_io_closed_p(@r_io).should == false
        @r_io.closed?.should == false
      end

      it "returns true when io is closed" do
        @r_io.close

        @o.rb_io_closed_p(@r_io).should == true
        @r_io.closed?.should == true
      end
    end

    quarantine! do # "Errno::EBADF: Bad file descriptor" at closing @r_io, @rw_io etc in the after :each hook
      describe "rb_io_open_descriptor" do
        it "creates a new IO instance" do
          io = @o.rb_io_open_descriptor(File, @r_io.fileno, 0, "a.txt", 60, "US-ASCII", "UTF-8", 0, {})
          io.should.is_a?(IO)
        end

        it "return an instance of the specified class" do
          io = @o.rb_io_open_descriptor(File, @r_io.fileno, 0, "a.txt", 60, "US-ASCII", "UTF-8", 0, {})
          io.class.should == File

          io = @o.rb_io_open_descriptor(IO, @r_io.fileno, 0, "a.txt", 60, "US-ASCII", "UTF-8", 0, {})
          io.class.should == IO
        end

        it "sets the specified file descriptor" do
          io = @o.rb_io_open_descriptor(File, @r_io.fileno, 0, "a.txt", 60, "US-ASCII", "UTF-8", 0, {})
          io.fileno.should == @r_io.fileno
        end

        it "sets the specified path" do
          io = @o.rb_io_open_descriptor(File, @r_io.fileno, 0, "a.txt", 60, "US-ASCII", "UTF-8", 0, {})
          io.path.should == "a.txt"
        end

        it "sets the specified mode" do
          io = @o.rb_io_open_descriptor(File, @r_io.fileno, CApiIOSpecs::FMODE_BINMODE, "a.txt", 60, "US-ASCII", "UTF-8", 0, {})
          io.should.binmode?

          io = @o.rb_io_open_descriptor(File, @r_io.fileno, CApiIOSpecs::FMODE_TEXTMODE, "a.txt", 60, "US-ASCII", "UTF-8", 0, {})
          io.should_not.binmode?
        end

        it "sets the specified timeout" do
          io = @o.rb_io_open_descriptor(File, @r_io.fileno, 0, "a.txt", 60, "US-ASCII", "UTF-8", 0, {})
          io.timeout.should == 60
        end

        it "sets the specified internal encoding" do
          io = @o.rb_io_open_descriptor(File, @r_io.fileno, 0, "a.txt", 60, "US-ASCII", "UTF-8", 0, {})
          io.internal_encoding.should == Encoding::US_ASCII
        end

        it "sets the specified external encoding" do
          io = @o.rb_io_open_descriptor(File, @r_io.fileno, 0, "a.txt", 60, "US-ASCII", "UTF-8", 0, {})
          io.external_encoding.should == Encoding::UTF_8
        end

        it "does not apply the specified encoding flags" do
          name = tmp("rb_io_open_descriptor_specs")
          File.write(name, "123\r\n456\n89")
          file = File.open(name, "r")

          io = @o.rb_io_open_descriptor(File, file.fileno, CApiIOSpecs::FMODE_READABLE, "a.txt", 60, "US-ASCII", "UTF-8", CApiIOSpecs::ECONV_UNIVERSAL_NEWLINE_DECORATOR, {})
          io.read_nonblock(20).should == "123\r\n456\n89"
        ensure
          file.close
          rm_r name
        end

        it "ignores the IO open options" do
          io = @o.rb_io_open_descriptor(File, @r_io.fileno, 0, "a.txt", 60, "US-ASCII", "UTF-8", 0, {external_encoding: "windows-1251"})
          io.external_encoding.should == Encoding::UTF_8

          io = @o.rb_io_open_descriptor(File, @r_io.fileno, 0, "a.txt", 60, "US-ASCII", "UTF-8", 0, {internal_encoding: "windows-1251"})
          io.internal_encoding.should == Encoding::US_ASCII

          io = @o.rb_io_open_descriptor(File, @r_io.fileno, 0, "a.txt", 60, "US-ASCII", "UTF-8", 0, {encoding: "windows-1251:binary"})
          io.external_encoding.should == Encoding::UTF_8
          io.internal_encoding.should == Encoding::US_ASCII

          io = @o.rb_io_open_descriptor(File, @r_io.fileno, 0, "a.txt", 60, "US-ASCII", "UTF-8", 0, {textmode: false})
          io.should_not.binmode?

          io = @o.rb_io_open_descriptor(File, @r_io.fileno, 0, "a.txt", 60, "US-ASCII", "UTF-8", 0, {binmode: true})
          io.should_not.binmode?

          io = @o.rb_io_open_descriptor(File, @r_io.fileno, 0, "a.txt", 60, "US-ASCII", "UTF-8", 0, {autoclose: false})
          io.should.autoclose?

          io = @o.rb_io_open_descriptor(File, @r_io.fileno, 0, "a.txt", 60, "US-ASCII", "UTF-8", 0, {path: "a.txt"})
          io.path.should == "a.txt"
        end

        it "ignores the IO encoding options" do
          io = @o.rb_io_open_descriptor(File, @w_io.fileno, CApiIOSpecs::FMODE_WRITABLE, "a.txt", 60, "US-ASCII", "UTF-8", 0, {crlf_newline: true})

          io.write("123\r\n456\n89")
          io.flush

          @r_io.read_nonblock(20).should == "123\r\n456\n89"
        end

        it "allows wrong mode" do
          io = @o.rb_io_open_descriptor(File, @w_io.fileno, CApiIOSpecs::FMODE_READABLE, "a.txt", 60, "US-ASCII", "UTF-8", 0, {})
          io.should.is_a?(File)

          platform_is_not :windows do
            -> { io.read_nonblock(1) }.should raise_error(Errno::EBADF)
          end

          platform_is :windows do
            -> { io.read_nonblock(1) }.should raise_error(IO::EWOULDBLOCKWaitReadable)
          end
        end

        it "tolerates NULL as rb_io_encoding *encoding parameter" do
          io = @o.rb_io_open_descriptor_without_encoding(File, @r_io.fileno, 0, "a.txt", 60)
          io.should.is_a?(File)
        end

        it "deduplicates path String" do
          path = "a.txt".dup
          io = @o.rb_io_open_descriptor(File, @r_io.fileno, 0, path, 60, "US-ASCII", "UTF-8", 0, {})
          io.path.should_not equal(path)

          path = "a.txt".freeze
          io = @o.rb_io_open_descriptor(File, @r_io.fileno, 0, path, 60, "US-ASCII", "UTF-8", 0, {})
          io.path.should_not equal(path)
        end

        it "calls #to_str to convert a path to a String" do
          path = Object.new
          def path.to_str; "a.txt"; end

          io = @o.rb_io_open_descriptor(File, @r_io.fileno, 0, path, 60, "US-ASCII", "UTF-8", 0, {})

          io.path.should == "a.txt"
        end
      end
    end
  end

  ruby_version_is "3.4" do
    describe "rb_io_maybe_wait" do
      it "returns nil if there is no error condition" do
        @o.rb_io_maybe_wait(0, @w_io, IO::WRITABLE, nil).should == nil
      end
    end
  end
end

describe "rb_fd_fix_cloexec" do
  before :each do
    @o = CApiIOSpecs.new

    @name = tmp("c_api_rb_io_specs")
    touch @name

    @io = new_io @name, "w:utf-8"
    @io.close_on_exec = false
    @io.sync = true
  end

  after :each do
    @io.close unless @io.closed?
    rm_r @name
  end

  it "sets close_on_exec on the IO" do
    @o.rb_fd_fix_cloexec(@io)
    @io.close_on_exec?.should be_true
  end

end

describe "rb_cloexec_open" do
  before :each do
    @o = CApiIOSpecs.new
    @name = tmp("c_api_rb_io_specs")
    touch @name

    @io = nil
  end

  after :each do
    @io.close unless @io.nil? || @io.closed?
    rm_r @name
  end

  it "sets close_on_exec on the newly-opened IO" do
    @io = @o.rb_cloexec_open(@name, 0, 0)
    @io.close_on_exec?.should be_true
  end
end

describe "rb_io_t modes flags" do
  before :each do
    @o = CApiIOSpecs.new
    @name = tmp("c_api_rb_io_specs")
    touch @name
  end

  after :each do
    rm_r @name
  end

  it "has the sync flag set if the IO object is synced in Ruby" do
    File.open(@name) { |io|
      io.sync = true
      @o.rb_io_mode_sync_flag(io).should == true
    }
  end

  it "has the sync flag unset if the IO object is not synced in Ruby" do
    File.open(@name) { |io|
      io.sync = false
      @o.rb_io_mode_sync_flag(io).should == false
    }
  end
end
