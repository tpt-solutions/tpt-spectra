name: Bug report
description: Report a defect in TPT Spectra
title: "[BUG] "
labels: [bug]
body:
  - type: textarea
    id: what
    attributes:
      label: What happened?
      description: A clear and concise description of the bug.
    validations:
      required: true
  - type: textarea
    id: reproduce
    attributes:
      label: Steps to reproduce
      description: How can we reproduce the issue?
    validations:
      required: true
  - type: textarea
    id: expected
    attributes:
      label: Expected behavior
    validations:
      required: true
  - type: input
    id: version
    attributes:
      label: Spectra version / commit
    validations:
      required: true
  - type: dropdown
    id: modality
    attributes:
      label: Modality (if applicable)
      options: [CT, MRI, Ultrasound, PET, DICOM parsing, AI bridge, CLI, Other]
