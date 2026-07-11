/**
 * Oracle resolver — maps Oracle JDBC raw physical types to aqua logical DataType.
 */

module.exports = {
  type: 'oracle',

  resolve: function (column) {
    var raw = column.rawType.toUpperCase();
    var length = column.length;
    var precision = column.precision;
    var scale = column.scale;

    // VARCHAR2 / VARCHAR → VARCHAR(n)
    if (raw === 'VARCHAR2' || raw === 'VARCHAR') {
      return { dataType: 'VARCHAR', length: length > 0 ? length : undefined };
    }

    // CHAR → VARCHAR(n)
    if (raw === 'CHAR') {
      return { dataType: 'VARCHAR', length: length > 0 ? length : undefined };
    }

    // CLOB / NCLOB → CLOB
    if (raw === 'CLOB' || raw === 'NCLOB') {
      return { dataType: 'CLOB' };
    }

    // NUMBER(p,0) → p≤10:INT, p>10:LONG; NUMBER(p,s) s>0 → DECIMAL(p,s)
    if (raw === 'NUMBER') {
      var p = precision != null && precision > 0 ? precision : undefined;
      var s = scale != null && scale >= 0 ? scale : 0;

      if (s > 0) {
        return { dataType: 'DECIMAL', precision: p, scale: s };
      }
      if (p != null && p > 10) {
        return { dataType: 'LONG' };
      }
      return { dataType: 'INT' };
    }

    // DATE → DATE
    if (raw === 'DATE') {
      return { dataType: 'DATE' };
    }

    // TIMESTAMP(…) → DATETIME
    if (raw.indexOf('TIMESTAMP') === 0) {
      return { dataType: 'DATETIME' };
    }

    // BLOB / RAW → BLOB
    if (raw === 'BLOB' || raw === 'RAW' || raw === 'LONG RAW') {
      return { dataType: 'BLOB' };
    }

    return null;
  }
};
