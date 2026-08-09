# frozen_string_literal: true

require "open3"
require "optparse"
require "set"

REQUIRED_FILES = %w[
  README.md
  README.zh-CN.md
  LICENSE
  CONTRIBUTING.md
  CONTRIBUTING.zh-CN.md
  SECURITY.md
  SECURITY.zh-CN.md
  SUPPORT.md
  SUPPORT.zh-CN.md
  CHANGELOG.md
  MAINTAINERS.md
  .github/CODEOWNERS
  docs/PRODUCT_SCOPE.md
  docs/PRODUCT_SCOPE.zh-CN.md
  docs/CONTRACT.md
  docs/CONTRACT.zh-CN.md
  docs/MATURITY.md
  docs/MATURITY.zh-CN.md
  docs/RELEASE_CHECKLIST.md
  docs/RELEASE_CHECKLIST.zh-CN.md
].freeze

BILINGUAL_PAIRS = [
  %w[README.md README.zh-CN.md],
  %w[CONTRIBUTING.md CONTRIBUTING.zh-CN.md],
  %w[SECURITY.md SECURITY.zh-CN.md],
  %w[SUPPORT.md SUPPORT.zh-CN.md],
  %w[docs/PRODUCT_SCOPE.md docs/PRODUCT_SCOPE.zh-CN.md],
  %w[docs/CONTRACT.md docs/CONTRACT.zh-CN.md],
  %w[docs/MATURITY.md docs/MATURITY.zh-CN.md],
  %w[docs/RELEASE_CHECKLIST.md docs/RELEASE_CHECKLIST.zh-CN.md]
].freeze

TEXT_EXTENSIONS = %w[
  .css .html .js .json .jsonc .lock .md .mjs .rb .rs .toml .yaml .yml
].freeze
TEXT_FILENAMES = %w[.gitignore LICENSE].freeze
UTF8_BOM = "\xEF\xBB\xBF".b.freeze
FORBIDDEN_PUBLIC_TEXT = Regexp.new(
  ["ag", "ent", "(?:[\\s_-]*)", "com", "mons"].join,
  Regexp::IGNORECASE
)

options = { root: Dir.pwd }
OptionParser.new do |parser|
  parser.on("--root PATH") { |path| options[:root] = path }
end.parse!

root = File.expand_path(options[:root])
errors = []
stdout, stderr, status = Open3.capture3(
  "git", "-C", root, "ls-files", "--cached", "--others", "--exclude-standard", "-z"
)
unless status.success?
  warn "Unable to list repository files: #{stderr.strip}"
  exit 1
end
repository_files = stdout.split("\0").reject(&:empty?).to_set

REQUIRED_FILES.each do |path|
  unless repository_files.include?(path) && File.file?(File.join(root, path))
    errors << "Missing required file: #{path}"
  end
end

BILINGUAL_PAIRS.each do |english, chinese|
  english_path = File.join(root, english)
  chinese_path = File.join(root, chinese)
  english_exists = repository_files.include?(english) && File.file?(english_path)
  chinese_exists = repository_files.include?(chinese) && File.file?(chinese_path)
  errors << "Missing bilingual pair: #{english}" unless english_exists
  errors << "Missing bilingual pair: #{chinese}" unless chinese_exists
  next unless english_exists && chinese_exists

  english_text = File.read(english_path, encoding: "UTF-8", invalid: :replace, undef: :replace)
  chinese_text = File.read(chinese_path, encoding: "UTF-8", invalid: :replace, undef: :replace)
  errors << "Missing Chinese entry link in #{english}" unless english_text.include?(File.basename(chinese))
  errors << "Missing English entry link in #{chinese}" unless chinese_text.include?(File.basename(english))
end

text_files = repository_files.select do |path|
  TEXT_EXTENSIONS.include?(File.extname(path).downcase) ||
    TEXT_FILENAMES.include?(File.basename(path))
end

text_files.sort.each do |path|
  absolute_path = File.join(root, path)
  next unless File.file?(absolute_path)

  bytes = File.binread(absolute_path)
  errors << "UTF-8 BOM is not allowed: #{path}" if bytes.start_with?(UTF8_BOM)
  content = bytes.force_encoding(Encoding::UTF_8)
  if content.valid_encoding?
    errors << "Legacy organization reference is forbidden: #{path}" if content.match?(FORBIDDEN_PUBLIC_TEXT)
  else
    errors << "Invalid UTF-8: #{path}"
  end
rescue SystemCallError => error
  errors << "Unable to read #{path}: #{error.message}"
end

if errors.empty?
  puts "Documentation checks passed (#{text_files.length} repository text files scanned)."
  exit 0
end

errors.each { |error| warn error }
exit 1
