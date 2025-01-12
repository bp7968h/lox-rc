# Grammer in lox
Below are the rules for grammer in [extended Backus-Naur form (EBNF)](https://en.wikipedia.org/wiki/Extended_Backus-Naur_form):

## Statement
```ebnf
statement := exprStmt
           | forStmt
           | ifStmt
           | printStmt
           | returnStmt
           | whileStmt
           | block ;
```

## Declaration
```ebnf
declaration := classDecl
             | funDecl
             | varDecl
             | statement ;
```               