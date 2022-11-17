# NixOS VM tests

## Run a single test

``` console
$ just run-test kuutamod
```


## Run a single test

This will open up a repl for interactive debugging

``` console
$ just debug-test kuutamod
# Inside the repl:
# starts all virtual machines
>>> start_all()
# opens up a shell to vm 'machine'
>>> machine.shell_interact()
```
