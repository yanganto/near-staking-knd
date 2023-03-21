# Reset neard

Sometimes it's desirable to reset neard, i.e. after a network fork or when
changing the chain for the current machine.
All the data related to kneard/neard is stored in `/var/lib/neard`.
Assuming that validator key and the validator node key as specified in
`kuutamo.kneard.validatorKeyFile` and `kuutamo.kneard.validatorNodeKeyFile`
is **NOT** stored in `/var/lib/neard`, it's safe to delete `/var/lib/neard`
and let the `kneard.service` restore configuration and the near chain data
from the backup:

```console
$ systemctl stop kneard
$ rm -rf /var/lib/neard
$ systemctl start kneard
```
