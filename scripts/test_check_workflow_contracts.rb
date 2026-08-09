# frozen_string_literal: true

require "fileutils"
require "minitest/autorun"
require "open3"
require "rbconfig"
require "tmpdir"
require "yaml"

class CheckWorkflowContractsTest < Minitest::Test
  CHECKER = File.expand_path("check_workflow_contracts.rb", __dir__)
  COMMIT = "21145ce218263e3b30359bab0c748da4702f801b"

  def test_valid_tinkora_references_pass
    with_fixture do |root|
      result = run_checker(root)

      assert result[:status].success?, result[:output]
      assert_includes result[:output], "Reusable workflow contracts passed"
    end
  end

  def test_floating_reference_fails
    with_fixture(reference: "main") do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "@#{COMMIT}"
    end
  end

  def test_missing_wasm_job_fails
    with_fixture(include_wasm: false) do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "job wasm must use"
    end
  end

  def test_wasm_browser_smoke_is_required
    with_fixture(playwright_smoke: false) do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "must enable the Playwright WASM smoke test"
    end
  end

  def test_pages_deployment_requires_main
    with_fixture(pages_main_gate: false) do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "must restrict assembly and deployment to main"
    end
  end

  def test_pages_artifacts_include_run_attempt
    with_fixture(pages_run_attempt: false) do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "must include github.run_attempt"
    end
  end

  def test_pages_assembly_waits_for_every_release_gate
    with_fixture(pages_release_gates: false) do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "must wait for quality, documentation, and supply-chain"
    end
  end

  private

  def with_fixture(
    reference: COMMIT,
    include_wasm: true,
    pages_main_gate: true,
    pages_run_attempt: true,
    pages_release_gates: true,
    playwright_smoke: true
  )
    Dir.mktmpdir("workflow-contracts-") do |root|
      quality_jobs = {
        "rust" => {
          "uses" => "Tinkora/.github/.github/workflows/reusable-rust-quality.yml@#{reference}",
          "with" => { "msrv" => "1.85.0" }
        }
      }
      if include_wasm
        quality_jobs["wasm"] = {
          "uses" => "Tinkora/.github/.github/workflows/reusable-wasm-quality.yml@#{reference}",
          "with" => { "playwright-smoke" => playwright_smoke }
        }
      end
      write_workflow(root, ".github/workflows/quality.yml", quality_jobs)
      write_workflow(
        root,
        ".github/workflows/supply-chain.yml",
        "audit" => "Tinkora/.github/.github/workflows/reusable-supply-chain.yml@#{reference}"
      )
      write_pages_workflow(
        root,
        reference,
        pages_main_gate,
        pages_run_attempt,
        pages_release_gates
      )
      yield root
    end
  end

  def write_workflow(root, relative_path, jobs)
    absolute_path = File.join(root, relative_path)
    FileUtils.mkdir_p(File.dirname(absolute_path))
    document = {
      "name" => "Fixture",
      "jobs" => jobs.transform_values do |configuration|
        configuration.is_a?(Hash) ? configuration : { "uses" => configuration }
      end
    }
    File.write(absolute_path, YAML.dump(document), encoding: "UTF-8")
  end

  def write_pages_workflow(root, reference, main_gate, run_attempt, release_gates)
    suffix = run_attempt ? "-${{ github.run_attempt }}" : ""
    condition = main_gate ? "github.ref == 'refs/heads/main'" : "success()"
    document = {
      "name" => "Fixture Pages",
      "jobs" => {
        "assemble" => {
          "if" => condition,
          "needs" => release_gates ? %w[quality documentation supply-chain] : ["quality"],
          "steps" => [
            { "with" => { "name" => "wasm-package-${{ github.run_id }}#{suffix}" } },
            { "with" => { "name" => "pages-source-${{ github.run_id }}#{suffix}" } }
          ]
        },
        "deploy" => {
          "if" => condition,
          "uses" => "Tinkora/.github/.github/workflows/reusable-pages.yml@#{reference}",
          "with" => { "source-artifact-name" => "pages-source-${{ github.run_id }}#{suffix}" }
        }
      }
    }
    path = File.join(root, ".github/workflows/pages.yml")
    FileUtils.mkdir_p(File.dirname(path))
    File.write(path, YAML.dump(document), encoding: "UTF-8")
  end

  def run_checker(root)
    stdout, stderr, status = Open3.capture3(RbConfig.ruby, CHECKER, "--root", root)
    { output: stdout + stderr, status: status }
  end
end
