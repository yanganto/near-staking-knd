# Reset neard

Sometimes it's desirable to reset neard, i.e. after a network fork or when
changing the chain for the current machine.
All the data related to kuutamod/neard is stored in `/var/lib/neard`. 
Assuming that validator key and the validator node key as specified in
`kuutamo.kuutamod.validatorKeyFile` and `kuutamo.kuutamod.validatorNodeKeyFile`
is **NOT** stored in `/var/lib/neard`, it's safe to delete `/var/lib/neard`
and let the `kuutamod.service` restore configuration and the near chain data
from the backup:

```console
$ systemctl stop kuutamod
$ rm -rf /var/lib/neard
$ systemctl start kuutamod
```
