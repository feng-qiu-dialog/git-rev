git-rev
=======

The `git-rev` is a command-line utitily to render template with git related info. It's inspired by the command `SubWCRev` from TortoiseSVN.

Handlebar template engine
--------------------------
The utility uses handlebar as its template engine. For how to use handlebars, please check [handlebars-rust](https://github.com/sunng87/handlebars-rust).

Access environment variables
----------------------------
Environment variables can be accessed through `env` object.

E.g. to put Environment Variable `SHELL` into template, you can write `{{ env.SHELL }}`.

Pass in extra variables
-----------------------
You can also pass in extra variables and access them through `extra` object.

E.g. to put the extra variable `extra_var` into template, you can write `{{ extra.extra_var }}`.

Handlebar helper
-----------------
A handlebar helper `git_log_format` is defined to help you get some more information out of git log.
`git_log_format` passes argument to git log like this `git log -1 --format={ YOUR ARGUMENTS }`. 

For more information on format, please check [here](https://git-scm.com/docs/pretty-formats).

Examples
--------

Example template file.

```
{
    "revision": "{{ revision }}",
    "rev_short": "{{ rev_short }}",
    "branch": "{{ branch }}",
    "commit_date": "{{ git_log_format "%aI" }}",
    "tag": "{{ tags.0 }}",
    "tags": [
        {{#each tags}}
        "{{this}}"
        {{/each}}
    ]
}
```

Basic usage
```
git-rev template.hbs -o output 
```

Filter tags
```
git-rev -t v* template.hbs -o output
```

Pass in extra variables
```
REM In Windows Cmd
git-rev template.hbs -o output -e "{ \"key\": \"value\" }"
```
```
# In Windows Powershell
git-rev template.hbs -o output -e '{ """key""": """value""" }'
```
```
# In *NIX Shell
git-rev template.hbs -o output -e '{ "key": "value" }'
```

Debug mode (prints out JSON object fed to the template)
```
git-rev template.hbs -o output -d
```
