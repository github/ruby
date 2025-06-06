# frozen_string_literal: true

module Bundler
  class CLI::Info
    attr_reader :gem_name, :options
    def initialize(options, gem_name)
      @options = options
      @gem_name = gem_name
    end

    def run
      Bundler.ui.silence do
        Bundler.definition.validate_runtime!
        Bundler.load.lock
      end

      spec = spec_for_gem(gem_name)

      if spec
        return print_gem_path(spec) if @options[:path]
        return print_gem_version(spec) if @options[:version]
        print_gem_info(spec)
      end
    end

    private

    def spec_for_gem(name)
      Bundler::CLI::Common.select_spec(name, :regex_match)
    end

    def print_gem_version(spec)
      Bundler.ui.info spec.version.to_s
    end

    def print_gem_path(spec)
      name = spec.name
      if name == "bundler"
        path = File.expand_path("../../..", __dir__)
      else
        path = spec.full_gem_path
        if spec.installation_missing?
          return Bundler.ui.warn "The gem #{name} is missing. It should be installed at #{path}, but was not found"
        end
      end

      Bundler.ui.info path
    end

    def print_gem_info(spec)
      metadata = spec.metadata
      name = spec.name
      gem_info = String.new
      gem_info << "  * #{name} (#{spec.version}#{spec.git_version})\n"
      gem_info << "\tSummary: #{spec.summary}\n" if spec.summary
      gem_info << "\tHomepage: #{spec.homepage}\n" if spec.homepage
      gem_info << "\tDocumentation: #{metadata["documentation_uri"]}\n" if metadata.key?("documentation_uri")
      gem_info << "\tSource Code: #{metadata["source_code_uri"]}\n" if metadata.key?("source_code_uri")
      gem_info << "\tFunding: #{metadata["funding_uri"]}\n" if metadata.key?("funding_uri")
      gem_info << "\tWiki: #{metadata["wiki_uri"]}\n" if metadata.key?("wiki_uri")
      gem_info << "\tChangelog: #{metadata["changelog_uri"]}\n" if metadata.key?("changelog_uri")
      gem_info << "\tBug Tracker: #{metadata["bug_tracker_uri"]}\n" if metadata.key?("bug_tracker_uri")
      gem_info << "\tMailing List: #{metadata["mailing_list_uri"]}\n" if metadata.key?("mailing_list_uri")
      gem_info << "\tPath: #{spec.full_gem_path}\n"
      gem_info << "\tDefault Gem: yes\n" if spec.respond_to?(:default_gem?) && spec.default_gem?
      gem_info << "\tReverse Dependencies: \n\t\t#{gem_dependencies.join("\n\t\t")}" if gem_dependencies.any?

      if name != "bundler" && spec.installation_missing?
        return Bundler.ui.warn "The gem #{name} is missing. Gemspec information is still available though:\n#{gem_info}"
      end

      Bundler.ui.info gem_info
    end

    def gem_dependencies
      @gem_dependencies ||= Bundler.definition.specs.filter_map do |spec|
        dependency = spec.dependencies.find {|dep| dep.name == gem_name }
        next unless dependency
        "#{spec.name} (#{spec.version}) depends on #{gem_name} (#{dependency.requirements_list.join(", ")})"
      end.sort
    end
  end
end
