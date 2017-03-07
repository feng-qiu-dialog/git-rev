git-rev
=======

The `git-rev` is a command-line utitily to render template with git related info. It's inspired by the command SubWCRev

Handlerbar Template Engine
--------------------------
The utility uses handlebar as its template engine. For how to use handlebars, please check [handlebars-rust](https://github.com/sunng87/handlebars-rust).


Examples
--------

Example Template File
```
{
    "revision": "{{ revision }}",
    "branch": "{{ branch }}",
    "tags": [
        {{#each tags}}
        "{{this}}"
        {{/each}}
    ]
}
```

Basic Usage
```
git-rev template.hbs output 
```

Filter Tags
```
git-rev -t v* template.hbs output
```