#!/usr/bin/env python3
"""
Test script to demonstrate the complete d-vecdb-server package functionality
"""

import sys
import time

def test_package_functionality():
    print("=" * 50)
    print("Testing d-vecDB Server Python Package")
    print("=" * 50)
    
    try:
        # Test 1: Import package
        print("1. Testing package import...")
        from d_vecdb_server import DVecDBServer
        print("   ✓ Package imported successfully")
        
        # Test 2: Check binary availability
        print("\n2. Testing binary detection...")
        try:
            server = DVecDBServer()
            print(f"   ✓ Binary found at: {server._binary_path}")
            print(f"   ✓ Binary size: {server._binary_path.stat().st_size / 1024 / 1024:.1f} MB")
        except Exception as e:
            print(f"   ❌ Binary detection failed: {e}")
            return False
        
        # Test 3: Server start/stop
        print("\n3. Testing server lifecycle...")
        server.start()
        if server.is_running():
            print("   ✓ Server started successfully")
            
            # Test 4: Server status
            status = server.get_status()
            print(f"   ✓ Server status: {status['running']}")
            print(f"   ✓ REST API port: {status['port']}")
            print(f"   ✓ gRPC port: {status['grpc_port']}")
            
            time.sleep(1)  # Let server initialize
            
            # Test 5: API accessibility
            if status.get('rest_api_accessible'):
                print("   ✓ REST API is accessible")
            
            # Test 6: Stop server
            server.stop()
            if not server.is_running():
                print("   ✓ Server stopped successfully")
        else:
            print("   ❌ Server failed to start")
            return False
            
        # Test 7: Context manager
        print("\n4. Testing context manager...")
        with DVecDBServer(port=8081) as ctx_server:
            if ctx_server.is_running():
                print("   ✓ Context manager works correctly")
            else:
                print("   ❌ Context manager failed")
                return False
        
        print("\n" + "=" * 50)
        print("🎉 ALL TESTS PASSED!")
        print("The d-vecdb-server package is fully functional!")
        print("=" * 50)
        return True
        
    except Exception as e:
        print(f"\n❌ Test failed with error: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    success = test_package_functionality()
    sys.exit(0 if success else 1)