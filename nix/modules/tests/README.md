# NixOS VM tests

## Run a single test

``` console
$ just run-test kneard
```


## Run a single test

This will open up a repl for interactive debugging

``` console
$ just debug-test kneard
# Inside the repl:
# starts all virtual machines
>>> start_all()
# opens up a shell to vm 'machine'
>>> machine.shell_interact()
```
