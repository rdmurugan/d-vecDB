class DVecdb < Formula
  desc "High-performance vector database written in Rust"
  homepage "https://github.com/rdmurugan/d-vecDB"
  url "https://github.com/rdmurugan/d-vecDB/archive/v0.1.4.tar.gz"
  sha256 "PLACEHOLDER_SHA256_WILL_BE_CALCULATED"
  license "MIT"
  head "https://github.com/rdmurugan/d-vecDB.git", branch: "main"

  depends_on "rust" => :build
  depends_on "protobuf" => :build

  def install
    # Build the release version
    system "cargo", "build", "--release", "--bin", "vectordb-server"
    system "cargo", "build", "--release", "--bin", "vectordb-cli"
    
    # Install binaries
    bin.install "target/release/vectordb-server"
    bin.install "target/release/vectordb-cli"
    
    # Install configuration template
    etc.install "config.toml" => "d-vecdb/config.toml"
    
    # Create data directory
    (var/"d-vecdb").mkpath
  end

  def post_install
    # Ensure config directory exists
    (etc/"d-vecdb").mkpath
    
    # Copy default config if it doesn't exist
    config_file = etc/"d-vecdb/config.toml"
    unless config_file.exist?
      config_file.write default_config
    end
  end

  def default_config
    <<~EOS
      [server]
      host = "127.0.0.1"
      port = 8080
      grpc_port = 9090
      workers = #{Hardware::CPU.cores}

      [storage]
      data_dir = "#{var}/d-vecdb"
      wal_sync_interval = "1s"
      memory_map_size = "1GB"

      [index]
      hnsw_max_connections = 16
      hnsw_ef_construction = 200
      hnsw_max_layer = 16

      [monitoring]
      enable_metrics = true
      prometheus_port = 9091
      log_level = "info"
    EOS
  end

  service do
    run [opt_bin/"vectordb-server", "--config", etc/"d-vecdb/config.toml"]
    keep_alive true
    working_dir var/"d-vecdb"
    log_path var/"log/d-vecdb.log"
    error_log_path var/"log/d-vecdb.error.log"
  end

  test do
    # Test that the binary runs and shows help
    assert_match "vectordb-server", shell_output("#{bin}/vectordb-server --help")
    assert_match "vectordb-cli", shell_output("#{bin}/vectordb-cli --help")
    
    # Test version output
    assert_match version.to_s, shell_output("#{bin}/vectordb-server --version")
  end
end