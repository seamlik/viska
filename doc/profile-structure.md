# Filesystem Structure of an Account Profile

An account profile is a file tree that contains all data of an account.

- Root directory that stores all profiles
  - `0C88CF8B12C190651C4B98885D035D43F1E87C20ADC80B5ED439FF9C76FF2BE3` (Account ID)
    - `certificate.der`
    - `key.der`
    - `database`
      - `main.cblite2` (Main database file, extension depends on database engine)
