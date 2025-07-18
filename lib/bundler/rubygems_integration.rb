# frozen_string_literal: true

require "rubygems" unless defined?(Gem)

module Bundler
  class RubygemsIntegration
    require "monitor"

    EXT_LOCK = Monitor.new

    def initialize
      @replaced_methods = {}
    end

    def version
      @version ||= Gem.rubygems_version
    end

    def provides?(req_str)
      Gem::Requirement.new(req_str).satisfied_by?(version)
    end

    def build_args
      require "rubygems/command"
      Gem::Command.build_args
    end

    def build_args=(args)
      require "rubygems/command"
      Gem::Command.build_args = args
    end

    def set_target_rbconfig(path)
      Gem.set_target_rbconfig(path)
    end

    def loaded_specs(name)
      Gem.loaded_specs[name]
    end

    def mark_loaded(spec)
      if spec.respond_to?(:activated=)
        current = Gem.loaded_specs[spec.name]
        current.activated = false if current
        spec.activated = true
      end
      Gem.loaded_specs[spec.name] = spec
    end

    def validate(spec)
      Bundler.ui.silence { spec.validate_for_resolution }
    rescue Gem::InvalidSpecificationException => e
      error_message = "The gemspec at #{spec.loaded_from} is not valid. Please fix this gemspec.\n" \
        "The validation error was '#{e.message}'\n"
      raise Gem::InvalidSpecificationException.new(error_message)
    rescue Errno::ENOENT
      nil
    end

    def stub_set_spec(stub, spec)
      stub.instance_variable_set(:@spec, spec)
    end

    def path(obj)
      obj.to_s
    end

    def ruby_engine
      Gem.ruby_engine
    end

    def read_binary(path)
      Gem.read_binary(path)
    end

    def inflate(obj)
      Gem::Util.inflate(obj)
    end

    def gem_dir
      Gem.dir
    end

    def gem_bindir
      Gem.bindir
    end

    def user_home
      Gem.user_home
    end

    def gem_path
      Gem.path
    end

    def reset
      Gem::Specification.reset
    end

    def post_reset_hooks
      Gem.post_reset_hooks
    end

    def suffix_pattern
      Gem.suffix_pattern
    end

    def gem_cache
      gem_path.map {|p| File.expand_path("cache", p) }
    end

    def spec_cache_dirs
      @spec_cache_dirs ||= begin
        dirs = gem_path.map {|dir| File.join(dir, "specifications") }
        dirs << Gem.spec_cache_dir
        dirs.uniq.select {|dir| File.directory? dir }
      end
    end

    def marshal_spec_dir
      Gem::MARSHAL_SPEC_DIR
    end

    def clear_paths
      Gem.clear_paths
    end

    def bin_path(gem, bin, ver)
      Gem.bin_path(gem, bin, ver)
    end

    def loaded_gem_paths
      loaded_gem_paths = Gem.loaded_specs.map {|_, s| s.full_require_paths }
      loaded_gem_paths.flatten
    end

    def ui=(obj)
      Gem::DefaultUserInteraction.ui = obj
    end

    def ext_lock
      EXT_LOCK
    end

    def spec_from_gem(path)
      require "rubygems/package"
      Gem::Package.new(path).spec
    end

    def build_gem(gem_dir, spec)
      build(spec)
    end

    def security_policy_keys
      %w[High Medium Low AlmostNo No].map {|level| "#{level}Security" }
    end

    def security_policies
      @security_policies ||= begin
        require "rubygems/security"
        Gem::Security::Policies
      rescue LoadError, NameError
        {}
      end
    end

    def reverse_rubygems_kernel_mixin
      # Disable rubygems' gem activation system
      if Gem.respond_to?(:discover_gems_on_require=)
        Gem.discover_gems_on_require = false
      else
        [::Kernel.singleton_class, ::Kernel].each do |k|
          if k.private_method_defined?(:gem_original_require)
            redefine_method(k, :require, k.instance_method(:gem_original_require))
          end
        end
      end
    end

    def replace_gem(specs_by_name)
      executables = nil

      [::Kernel.singleton_class, ::Kernel].each do |kernel_class|
        redefine_method(kernel_class, :gem) do |dep, *reqs|
          if executables&.include?(File.basename(caller_locations(1, 1).first.path))
            break
          end

          reqs.pop if reqs.last.is_a?(Hash)

          unless dep.respond_to?(:name) && dep.respond_to?(:requirement)
            dep = Gem::Dependency.new(dep, reqs)
          end

          if spec = specs_by_name[dep.name]
            return true if dep.matches_spec?(spec)
          end

          message = if spec.nil?
            target_file = begin
                            Bundler.default_gemfile.basename
                          rescue GemfileNotFound
                            "inline Gemfile"
                          end
            "#{dep.name} is not part of the bundle." \
            " Add it to your #{target_file}."
          else
            "can't activate #{dep}, already activated #{spec.full_name}. " \
            "Make sure all dependencies are added to Gemfile."
          end

          e = Gem::LoadError.new(message)
          e.name = dep.name
          e.requirement = dep.requirement
          raise e
        end
      end
    end

    # Used to give better error messages when activating specs outside of the current bundle
    def replace_bin_path(specs_by_name)
      redefine_method(gem_class, :find_spec_for_exe) do |gem_name, *args|
        exec_name = args.first
        raise ArgumentError, "you must supply exec_name" unless exec_name

        spec_with_name = specs_by_name[gem_name]
        matching_specs_by_exec_name = specs_by_name.values.select {|s| s.executables.include?(exec_name) }
        spec = matching_specs_by_exec_name.delete(spec_with_name)

        unless spec || !matching_specs_by_exec_name.empty?
          message = "can't find executable #{exec_name} for gem #{gem_name}"
          if spec_with_name.nil?
            message += ". #{gem_name} is not currently included in the bundle, " \
                       "perhaps you meant to add it to your #{Bundler.default_gemfile.basename}?"
          end
          raise Gem::Exception, message
        end

        unless spec
          spec = matching_specs_by_exec_name.shift
          warn \
            "Bundler is using a binstub that was created for a different gem (#{spec.name}).\n" \
            "You should run `bundle binstub #{gem_name}` " \
            "to work around a system/bundle conflict."
        end

        unless matching_specs_by_exec_name.empty?
          conflicting_names = matching_specs_by_exec_name.map(&:name).join(", ")
          warn \
            "The `#{exec_name}` executable in the `#{spec.name}` gem is being loaded, but it's also present in other gems (#{conflicting_names}).\n" \
            "If you meant to run the executable for another gem, make sure you use a project specific binstub (`bundle binstub <gem_name>`).\n" \
            "If you plan to use multiple conflicting executables, generate binstubs for them and disambiguate their names."
        end

        spec
      end
    end

    # Replace or hook into RubyGems to provide a bundlerized view
    # of the world.
    def replace_entrypoints(specs)
      specs_by_name = add_default_gems_to(specs)

      reverse_rubygems_kernel_mixin
      begin
        # bundled_gems only provide with Ruby 3.3 or later
        require "bundled_gems"
      rescue LoadError
      else
        Gem::BUNDLED_GEMS.replace_require(specs) if Gem::BUNDLED_GEMS.respond_to?(:replace_require)
      end
      replace_gem(specs_by_name)
      stub_rubygems(specs_by_name.values)
      replace_bin_path(specs_by_name)

      Gem.clear_paths
    end

    # Add default gems not already present in specs, and return them as a hash.
    def add_default_gems_to(specs)
      specs_by_name = specs.reduce({}) do |h, s|
        h[s.name] = s
        h
      end

      Bundler.rubygems.default_stubs.each do |stub|
        default_spec = stub.to_spec
        default_spec_name = default_spec.name
        next if specs_by_name.key?(default_spec_name)

        specs_by_name[default_spec_name] = default_spec
      end

      specs_by_name
    end

    def undo_replacements
      @replaced_methods.each do |(sym, klass), method|
        redefine_method(klass, sym, method)
      end
      post_reset_hooks.reject! {|proc| proc.binding.source_location[0] == __FILE__ }
      @replaced_methods.clear
    end

    def redefine_method(klass, method, unbound_method = nil, &block)
      visibility = method_visibility(klass, method)
      begin
        if (instance_method = klass.instance_method(method)) && method != :initialize
          # doing this to ensure we also get private methods
          klass.send(:remove_method, method)
        end
      rescue NameError
        # method isn't defined
        nil
      end
      @replaced_methods[[method, klass]] = instance_method
      if unbound_method
        klass.send(:define_method, method, unbound_method)
        klass.send(visibility, method)
      elsif block
        klass.send(:define_method, method, &block)
        klass.send(visibility, method)
      end
    end

    def method_visibility(klass, method)
      if klass.private_method_defined?(method)
        :private
      elsif klass.protected_method_defined?(method)
        :protected
      else
        :public
      end
    end

    def stub_rubygems(specs)
      Gem::Specification.all = specs

      Gem.post_reset do
        Gem::Specification.all = specs
      end

      redefine_method(gem_class, :finish_resolve) do |*|
        []
      end

      redefine_method(gem_class, :load_plugins) do |*|
        load_plugin_files specs.flat_map(&:plugins)
      end
    end

    def plain_specs
      Gem::Specification._all
    end

    def plain_specs=(specs)
      Gem::Specification.all = specs
    end

    def fetch_specs(remote, name, fetcher)
      require "rubygems/remote_fetcher"
      path = remote.uri.to_s + "#{name}.#{Gem.marshal_version}.gz"
      string = fetcher.fetch_path(path)
      specs = Bundler.safe_load_marshal(string)
      raise MarshalError, "Specs #{name} from #{remote} is expected to be an Array but was unexpected class #{specs.class}" unless specs.is_a?(Array)
      specs
    rescue Gem::RemoteFetcher::FetchError
      # it's okay for prerelease to fail
      raise unless name == "prerelease_specs"
    end

    def fetch_all_remote_specs(remote, gem_remote_fetcher)
      specs = fetch_specs(remote, "specs", gem_remote_fetcher)
      pres = fetch_specs(remote, "prerelease_specs", gem_remote_fetcher) || []

      specs.concat(pres)
    end

    def download_gem(spec, uri, cache_dir, fetcher)
      require "rubygems/remote_fetcher"
      uri = Bundler.settings.mirror_for(uri)
      redacted_uri = Gem::Uri.redact(uri)

      Bundler::Retry.new("download gem from #{redacted_uri}").attempts do
        gem_file_name = spec.file_name
        local_gem_path = File.join cache_dir, gem_file_name
        return if File.exist? local_gem_path

        begin
          remote_gem_path = uri + "gems/#{gem_file_name}"

          SharedHelpers.filesystem_access(local_gem_path) do
            fetcher.cache_update_path remote_gem_path, local_gem_path
          end
        rescue Gem::RemoteFetcher::FetchError
          raise if spec.original_platform == spec.platform

          original_gem_file_name = "#{spec.original_name}.gem"
          raise if gem_file_name == original_gem_file_name

          gem_file_name = original_gem_file_name
          retry
        end
      end
    rescue Gem::RemoteFetcher::FetchError => e
      raise Bundler::HTTPError, "Could not download gem from #{redacted_uri} due to underlying error <#{e.message}>"
    end

    def build(spec, skip_validation = false)
      require "rubygems/package"
      Gem::Package.build(spec, skip_validation)
    end

    def path_separator
      Gem.path_separator
    end

    def all_specs
      SharedHelpers.major_deprecation 2, "Bundler.rubygems.all_specs has been removed in favor of Bundler.rubygems.installed_specs"

      Gem::Specification.stubs.map do |stub|
        StubSpecification.from_stub(stub)
      end
    end

    def installed_specs
      Gem::Specification.stubs.reject(&:default_gem?).map do |stub|
        StubSpecification.from_stub(stub)
      end
    end

    def default_specs
      Gem::Specification.default_stubs.map do |stub|
        StubSpecification.from_stub(stub)
      end
    end

    def find_bundler(version)
      find_name("bundler").find {|s| s.version.to_s == version }
    end

    def find_name(name)
      Gem::Specification.stubs_for(name).map(&:to_spec)
    end

    def default_stubs
      Gem::Specification.default_stubs("*.gemspec")
    end

    private

    def gem_class
      class << Gem; self; end
    end
  end

  def self.rubygems
    @rubygems ||= RubygemsIntegration.new
  end
end
