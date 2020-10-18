function (doc) {
  if (doc.manifest && doc.reference && !doc.reference.startsWith("sha256:") && !doc.reference.startsWith("sha512:")) {
    emit(doc.image, doc.reference);
  }
}
