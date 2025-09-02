class DVecdbBin < Formula
  desc "High-performance vector database written in Rust (binary release)"
  homepage "https://github.com/rdmurugan/d-vecDB"
  version "0.1.1"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/rdmurugan/d-vecDB/releases/download/v0.1.1/vectordb-server-macos-arm64"
      sha256 "PLACEHOLDER_ARM64_SHA256"
    else
      url "https://github.com/rdmurugan/d-vecDB/releases/download/v0.1.1/vectordb-server-macos-x64"
      sha256 "PLACEHOLDER_X64_SHA256"
    end
  end

  on_linux do
    url "https://github.com/rdmurugan/d-vecDB/releases/download/v0.1.1/vectordb-server-linux-musl-x64"
    sha256 "PLACEHOLDER_LINUX_SHA256"
  end

  def install
    # The downloaded file is the binary itself
    if Hardware::CPU.arm?
      binary_name = "vectordb-server-macos-arm64"
    elsif OS.mac?
      binary_name = "vectordb-server-macos-x64"
    else
      binary_name = "vectordb-server-linux-musl-x64"
    end

    # Install the binary
    bin.install binary_name => "vectordb-server"
    
    # Create configuration template
    (etc/"d-vecdb").mkpath
    (var/"d-vecdb").mkpath
    
    # Install default config
    config_file = etc/"d-vecdb/config.toml"
    config_file.write default_config
  end

  def default_config
    <<~EOS
      # d-vecDB Configuration File
      # Installed via Homebrew

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

      [performance]
      batch_size = 500
      insert_workers = #{[Hardware::CPU.cores / 2, 2].max}
      query_cache_size = "200MB"
    EOS
  end

  service do
    run [opt_bin/"vectordb-server", "--config", etc/"d-vecdb/config.toml"]
    keep_alive true
    working_dir var/"d-vecdb"
    log_path var/"log/d-vecdb.log"
    error_log_path var/"log/d-vecdb.error.log"
    environment_variables RUST_LOG: "info"
  end

  def caveats
    <<~EOS
      d-vecDB has been installed with Homebrew!

      Configuration file: #{etc}/d-vecdb/config.toml
      Data directory: #{var}/d-vecdb
      Log files: #{var}/log/d-vecdb.log

      To start d-vecDB now:
        vectordb-server --config #{etc}/d-vecdb/config.toml

      To start d-vecDB as a service:
        brew services start d-vecdb-bin

      To stop the service:
        brew services stop d-vecdb-bin

      Python client installation:
        pip install d-vecdb

      Documentation: https://github.com/rdmurugan/d-vecDB#readme
    EOS
  end

  test do
    # Test that the binary runs and shows help
    assert_match "vectordb-server", shell_output("#{bin}/vectordb-server --help")
    
    # Test version output  
    version_output = shell_output("#{bin}/vectordb-server --version 2>&1")
    assert_match(/\d+\.\d+\.\d+/, version_output)
    
    # Test config file exists
    assert_predicate etc/"d-vecdb/config.toml", :exist?
  end
end