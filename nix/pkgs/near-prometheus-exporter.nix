{ buildGoModule, fetchFromGitHub, lib }:

buildGoModule {
  pname = "near-prometheus-exporter";
  version = "2022-11-01";

  src = fetchFromGitHub {
    owner = "kuutamolabs";
    repo = "near-prometheus-exporter";
    rev = "180f5a9c3a3e5316465bc7ff6f1767476a8387ea";
    sha256 = "sha256-0Cdu2QY2UKYuRASY4NbGzir8+UwpNDR/VAme+6dvfLE=";
  };

  vendorSha256 = "sha256-RpEc062ObLv7ozkZz4TUOfk/SFFs84RvqALBZYjqU3k=";

  meta = with lib; {
    description = "Exports various metrics from Near node for consumption by Prometheus.";
    homepage = "https://github.com/masknetgoal634/near-prometheus-exporter";
    license = licenses.mit;
    maintainers = with maintainers; [ mic92 ];
    platforms = platforms.unix;
  };
}
