# frozen_string_literal: true

require "fileutils"
require "minitest/autorun"
require "open3"
require "rbconfig"
require "tmpdir"

class AssemblePagesTest < Minitest::Test
  ASSEMBLER = File.expand_path("assemble_pages.rb", __dir__)

  def test_assembles_static_assets_wasm_and_schema
    with_fixture do |root, wasm_package|
      result = run_assembler(root, wasm_package)

      assert result[:status].success?, result[:output]
      assert_equal "<!doctype html>\n<title>DMG Background</title>\n",
        read(root, "dist/index.html")
      assert_equal "body {}\n", read(root, "dist/styles.css")
      assert_equal "export default {};\n", read(root, "dist/pkg/dmg_background_web.js")
      assert File.file?(File.join(root, "dist/pkg/dmg_background_web_bg.wasm"))
      assert_equal "{\"$id\":\"layout-v1\"}\n", read(root, "dist/schema/dmg-layout-v1.json")
      refute File.exist?(File.join(root, "dist/pkg/.gitignore"))
      refute File.exist?(File.join(root, "dist/sentinel.txt"))
    end
  end

  def test_rejects_unknown_wasm_files_without_replacing_previous_site
    with_fixture do |root, wasm_package|
      File.write(File.join(wasm_package, "unexpected.txt"), "no\n", encoding: "UTF-8")

      result = run_assembler(root, wasm_package)

      refute result[:status].success?
      assert_includes result[:output], "unexpected file"
      assert_equal "previous\n", read(root, "dist/sentinel.txt")
    end
  end

  def test_rejects_wasm_symlinks_without_replacing_previous_site
    with_fixture do |root, wasm_package|
      File.symlink(
        "dmg_background_web.js",
        File.join(wasm_package, "dmg_background_web.d.ts")
      )

      result = run_assembler(root, wasm_package)

      refute result[:status].success?
      assert_includes result[:output], "WASM package contains a symbolic link"
      assert_equal "previous\n", read(root, "dist/sentinel.txt")
    end
  end

  def test_rejects_static_symlinks_without_replacing_previous_site
    with_fixture do |root, wasm_package|
      File.symlink("index.html", File.join(root, "crates/dmg_background_web/static/linked.html"))

      result = run_assembler(root, wasm_package)

      refute result[:status].success?
      assert_includes result[:output], "Static UI contains a symbolic link"
      assert_equal "previous\n", read(root, "dist/sentinel.txt")
    end
  end

  def test_rejects_a_symlinked_schema
    with_fixture do |root, wasm_package|
      schema = File.join(root, "crates/dmg_background_core/schema/dmg_layout.schema.json")
      FileUtils.rm(schema)
      File.write(File.join(root, "outside-schema.json"), "{}\n", encoding: "UTF-8")
      File.symlink(File.join(root, "outside-schema.json"), schema)

      result = run_assembler(root, wasm_package)

      refute result[:status].success?
      assert_includes result[:output], "Schema must be a real file"
      assert_equal "previous\n", read(root, "dist/sentinel.txt")
    end
  end

  def test_rejects_a_symlinked_output_directory
    with_fixture do |root, wasm_package|
      FileUtils.rm_r(File.join(root, "dist"))
      FileUtils.mkdir_p(File.join(root, "outside"))
      File.symlink(File.join(root, "outside"), File.join(root, "dist"))

      result = run_assembler(root, wasm_package)

      refute result[:status].success?
      assert_includes result[:output], "dist must be a real directory"
      assert_empty Dir.children(File.join(root, "outside"))
    end
  end

  private

  def with_fixture
    Dir.mktmpdir("assemble-pages-") do |root|
      static = File.join(root, "crates/dmg_background_web/static")
      schema = File.join(root, "crates/dmg_background_core/schema")
      wasm_package = File.join(root, "wasm-package")
      FileUtils.mkdir_p(static)
      FileUtils.mkdir_p(schema)
      FileUtils.mkdir_p(wasm_package)
      FileUtils.mkdir_p(File.join(root, "dist"))
      File.write(File.join(root, "dist/sentinel.txt"), "previous\n", encoding: "UTF-8")
      File.write(
        File.join(static, "index.html"),
        "<!doctype html>\n<title>DMG Background</title>\n",
        encoding: "UTF-8"
      )
      File.write(File.join(static, "styles.css"), "body {}\n", encoding: "UTF-8")
      FileUtils.mkdir_p(File.join(static, "pkg"))
      File.write(File.join(static, "pkg/stale.js"), "stale\n", encoding: "UTF-8")
      File.write(
        File.join(schema, "dmg_layout.schema.json"),
        "{\"$id\":\"layout-v1\"}\n",
        encoding: "UTF-8"
      )
      File.write(File.join(wasm_package, "package.json"), "{}\n", encoding: "UTF-8")
      File.write(
        File.join(wasm_package, "dmg_background_web.js"),
        "export default {};\n",
        encoding: "UTF-8"
      )
      File.binwrite(File.join(wasm_package, "dmg_background_web_bg.wasm"), "\0asm")
      File.write(File.join(wasm_package, ".gitignore"), "*\n", encoding: "UTF-8")
      yield root, wasm_package
    end
  end

  def run_assembler(root, wasm_package)
    stdout, stderr, status = Open3.capture3(
      RbConfig.ruby,
      ASSEMBLER,
      "--root",
      root,
      "--wasm-package",
      wasm_package
    )
    { output: stdout + stderr, status: status }
  end

  def read(root, path)
    File.read(File.join(root, path), encoding: "UTF-8")
  end
end
