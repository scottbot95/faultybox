{
  self,
  nixpkgs ? self.inputs.nixpkgs,
  ci-utils ? self.inputs.ci-utils,
}:
let 
  
in ci-utils.lib.mkNetwork {
  inherit nixpkgs;
  stateName = "faultybox.nixops";

  network.description = "FaultyBox Game server deployment to my Homelab";
  
  faultybox = { config, ... }: {
    imports = [
      self.nixosModules.faultybox
    ];

    deployment.proxmox = {
      cores = 8;
      memory = 8192;
      startOnBoot = true;
      disks = [{ 
        volume = "nvme0";
        size = "20G";
        enableSSDEmulation = true;
        enableDiscard = true;
      }];
      network = [{
        bridge = "vmbr0";
        tag = 20;
      }];
    };

    networking.hostName = "games";

    services.faultybox.enable = true;
    services.faultybox.openFirewall = true;
  };
}